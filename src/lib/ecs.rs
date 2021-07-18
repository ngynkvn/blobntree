use crate::HashMap;
use core::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::marker::PhantomData;

type SystemId = TypeId;
type EntityId = u64;

pub trait Component: Any {}

pub struct EntityBuilder<'a> {
    entity: Entity,
    world: &'a mut World,
}

impl<'a> EntityBuilder<'a> {
    pub fn with<C: Component>(&mut self, component: C) -> &mut Self {
        let component_array = self.world.components.get_mut::<EntityMap<C>>();
        if component_array.0.len() <= self.entity.index() {
            component_array
                .0
                .resize_with(self.entity.index() + 1, Default::default);
        }
        component_array.set(self.entity, component);
        self
    }
    pub fn build(&mut self) {
        self.world.entities[self.entity.index()] = self.entity;
    }
}

pub type Entity = GenerationalIndex;

#[derive(Eq, PartialEq, Clone, Copy, Default, Debug)]
pub struct GenerationalIndex {
    index: usize,
    generation: u64,
}

impl GenerationalIndex {
    pub fn index(&self) -> usize {
        self.index
    }
}
// Generational Index allocator to deal with handling ECS component data.
// https://kyren.github.io/2018/09/14/rustconf-talk.html
struct AllocatorEntry {
    is_live: bool,
    generation: u64,
}
#[derive(Default)]
pub struct GenerationalIndexAllocator {
    entries: Vec<AllocatorEntry>,
    free: Vec<usize>,
}
impl GenerationalIndexAllocator {
    pub fn allocate(&mut self) -> GenerationalIndex {
        if let Some(index) = self.free.pop() {
            self.entries[index].is_live = true;
            self.entries[index].generation += 1;
            GenerationalIndex {
                index,
                generation: self.entries[index].generation,
            }
        } else {
            let index = GenerationalIndex {
                index: self.entries.len(),
                generation: 0,
            };
            self.entries.push(AllocatorEntry {
                is_live: true,
                generation: 0,
            });
            index
        }
    }

    // Returns true if the index was allocated before and is now deallocated
    pub fn deallocate(&mut self, index: GenerationalIndex) -> bool {
        if let Some(entry) = self.entries.get_mut(index.index) {
            entry.is_live = false;
            self.free.push(index.index);
            true
        } else {
            false
        }
    }

    pub fn is_live(&self, index: GenerationalIndex) -> bool {
        self.entries
            .get(index.index)
            .map(|entry| entry.is_live)
            .unwrap_or(false)
    }
}

#[derive(Clone)]
struct ArrayEntry<T> {
    value: T,
    generation: u64,
}

// An associative array from GenerationalIndex to some Value T.
pub struct GenerationalIndexArray<T>(Vec<Option<ArrayEntry<T>>>);

impl<T> GenerationalIndexArray<T> {
    // Set the value for some generational index.  May overwrite past generation
    // values.
    pub fn set(&mut self, index: GenerationalIndex, value: T) {
        if let Some(mut entry) = self.0[index.index()].as_mut() {
            entry.value = value
        }
    }

    // Gets the value for some generational index, the generation must match.
    pub fn get(&self, index: GenerationalIndex) -> Option<&T> {
        self.0[index.index()]
            .as_ref()
            .filter(|entry| entry.generation == index.generation)
            .map(|entry| &entry.value)
    }
    pub fn get_mut(&mut self, index: GenerationalIndex) -> Option<&mut T> {
        self.0[index.index()]
            .as_mut()
            .filter(|entry| entry.generation == index.generation)
            .map(|entry| &mut entry.value)
    }
}

type EntityMap<T> = GenerationalIndexArray<T>;

#[derive(Default)]
pub struct ComponentMap(HashMap<TypeId, Box<dyn Any>>);

impl ComponentMap {
    fn get<T: Any>(&self) -> &T {
        self.0
            .get(&TypeId::of::<T>())
            .unwrap()
            .as_ref()
            .downcast_ref::<T>()
            .unwrap()
    }
    fn get_mut<T: Any>(&mut self) -> &mut T {
        self.0
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .as_mut()
            .downcast_mut::<T>()
            .unwrap()
    }
    fn insert<T: Any>(&mut self, t: T) {
        self.0.insert(TypeId::of::<T>(), Box::new(t));
    }
}

// World is a base container class that we can register components and entities to.
#[derive(Default)]
pub struct World {
    // components: ComponentMap,
    component_incr: usize,
    component_types: HashMap<TypeId, usize>,
    entities: Vec<Entity>,
    allocator: GenerationalIndexAllocator,
    components: ComponentMap,
}

// System

pub trait System<'a> {
    type SystemData: Join<'a>;
    fn update(&mut self, entities: Entities<'a, Self::SystemData>);
}

impl<'a> World {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn register<T: 'static>(&mut self) {
        self.component_types
            .insert(TypeId::of::<T>(), self.component_incr);
        self.components.insert(GenerationalIndexArray::<T>(vec![]));
        self.component_incr += 1;
    }

    pub fn create_entity(&mut self) -> EntityBuilder {
        let index = self.allocator.allocate();
        if self.entities.len() <= index.index() {
            self.entities
                .resize_with(index.index() + 1, Default::default);
        }
        EntityBuilder {
            entity: index,
            world: self,
        }
    }

    pub fn run_system<'b, S, D>(&'b mut self, system: &mut S)
    where
        D: Join<'a>,
        S: System<'a, SystemData = D>,
        'b: 'a,
    {
        system.update(Entities::<'a> {
            entities: &self.entities,
            components: &mut self.components,
            _m: PhantomData,
        })
    }
}

pub struct Entities<'a, D: Join<'a>> {
    pub entities: &'a Vec<Entity>,
    pub components: &'a mut ComponentMap,
    _m: PhantomData<D>,
}

pub fn join<'a, D: Join<'a>>(map: &'a mut ComponentMap, entity: Entity) -> Option<D::Type> {
    D::get(map, entity)
}


pub trait Join<'a> {
    type Type;
    fn get(map: &'a mut ComponentMap, entity: Entity) -> Option<Self::Type>;
}

impl<'a, T> Join<'a> for (T,) where
T: Component {
    type Type = &'a mut T;
    fn get(map: &'a mut ComponentMap, entity: Entity) -> Option<Self::Type> {
        map.get_mut::<EntityMap<T>>().get_mut(entity)
    }
}

impl<'a, A: Component, B: Component> Join<'a> for (A, B) {
    type Type = (&'a mut A, &'a mut B);
    fn get(map: &'a mut ComponentMap, entity: Entity) -> Option<Self::Type> {
        let result = unsafe {
            let a = map.get_mut::<EntityMap<A>>() as *mut EntityMap<A>;
            let a = a.as_mut().unwrap().get_mut(entity);
            let b = map.get_mut::<EntityMap<B>>() as *mut EntityMap<B>;
            let b = b.as_mut().unwrap().get_mut(entity);
            (a, b)
        };
            match result {
                (Some(a), Some(b)) => Some((a, b)),
                _ => None,
            }
        }
    }
