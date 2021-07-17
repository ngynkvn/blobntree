use crate::HashMap;
use crate::HashSet;
use core::any::Any;
use core::marker::PhantomData;
use core::mem::take;
use std::any::TypeId;
use std::iter::FromIterator;

type SystemId = TypeId;
type EntityId = u64;
type ComponentMap = HashMap<SystemId, Box<dyn Any>>;

pub trait Component: Any {}

pub struct EntityBuilder<'a> {
    entity: Entity,
    world: &'a mut World,
}

impl<'a> EntityBuilder<'a> {
    pub fn with<C: Component>(&mut self, component: C) -> &mut Self {
        let id = self
            .world
            .component_types
            .get(&component.type_id())
            .unwrap();
        self.world
            .components
            .resize_with(self.world.component_types.len(), Default::default);
        self.world.components[self.entity.index.index()][*id] = Some(Box::new(component));
        self
    }
    pub fn build(&mut self) {
        // let mut entity = take(&mut self.entity);
        // entity.component_types = HashSet::from_iter(entity.components.keys().cloned());
        self.world.entities[self.entity.index.index()] = self.entity;
    }
}

#[derive(Default)]
pub struct Entity {
    index: GenerationalIndex,
    mask: HashSet<TypeId>,
}

#[derive(Eq, PartialEq, Clone, Default)]
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
                index: self.entries.len() - 1,
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

// World is a base container class that we can register components and entities to.
#[derive(Default)]
pub struct World {
    // components: ComponentMap,
    component_incr: usize,
    component_types: HashMap<TypeId, usize>,
    entities: Vec<Entity>,
    allocator: GenerationalIndexAllocator,
    components: Vec<Vec<Option<Box<dyn Component>>>>,
}

// System

pub trait System {
    fn update<'a>(&mut self, entities: Entities<'a>);
}

impl<'a> World {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn register<T: 'static>(&mut self) {
        self.component_types
            .insert(TypeId::of::<T>(), self.component_incr);
        self.component_incr += 1;
    }

    pub fn create_entity(&mut self) -> EntityBuilder {
        let index = self.allocator.allocate();
        if self.entities.len() <= index.index() {
            self.entities
                .resize_with(index.index() + 1, Default::default);
            self.components
                .resize_with(index.index() + 1, Default::default);
        }
        EntityBuilder {
            entity: Default::default(),
            world: self,
        }
    }

    pub fn run_system<'b, S>(&'b mut self, system: &mut S, components: &[TypeId])
    where
        S: System,
    {
        let set = HashSet::from_iter(components.iter().cloned());
        let i = self.entities.iter();
        let mut matches = i.filter(|entity| entity.mask.is_superset(&set));
        let entities = Entities {
            world: self,
            iter: &mut matches,
        };
        system.update(entities)
    }
}

pub struct Entities<'a> {
    world: &'a mut World,
    iter: &'a mut Iterator<Item = &'a Entity>,
}

pub struct EntityRef<'a> {
    types: HashMap<TypeId, usize>,
    components: &'a mut Vec<Option<Box<dyn Component>>>,
}

impl<'a> EntityRef<'a> {
    pub fn get<T: Component>(&'a self) -> &'a T {
        let id = self.types.get(&TypeId::of::<T>()).unwrap();
        let component = self.components[*id].unwrap();
        let component = component.downcast_ref();
        component.unwrap()
    }

    // pub fn set<T: Component>(&'a mut self, value: T) {
    //     self.components.insert(TypeId::of::<T>(), Box::new(value));
    // }
}

impl<'a> Iterator for Entities<'a> {
    type Item = EntityRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|entity| EntityRef {
            types: self.world.component_types,
            components: self
                .world
                .components
                .get_unchecked_mut(entity.index.index()),
        })
    }
}
