use core::marker::PhantomData;
use crate::HashSet;
use core::mem::take;
use crate::HashMap;
use core::any::Any;
use std::any::TypeId;
use std::iter::FromIterator;

type SystemId = TypeId;
type EntityId = u64;
type ComponentMap = HashMap<SystemId, Box<dyn Any>>;

pub trait Component: Any {
}

impl Component for () {}

pub struct EntityBuilder<'a> {
    world: &'a mut World,
    entity: Entity,
}

impl<'a> EntityBuilder<'a> {
    pub fn with<C: Component>(&mut self, component: C) -> &mut Self {
        self.entity.components.insert(TypeId::of::<C>(), Box::new(component));
        self
    }
    pub fn build(&mut self) {
        let mut entity = take(&mut self.entity);
        entity.component_types = HashSet::from_iter(entity.components.keys().cloned());
        self.world.entities.push(entity);
    }
}

#[derive(Default)]
pub struct Entity {
    pub id: EntityId,
    pub component_types: HashSet<TypeId>,
    pub components: ComponentMap,
}

impl<'a> Entity {
    pub fn get<T: Component>(&'a self) -> &'a T {
        let component = self.components.get(&TypeId::of::<T>()).unwrap();
        let component = component.downcast_ref();
        component.unwrap()

    }
}

// World is a base container class that we can register components and entities to.
#[derive(Default)]
pub struct World {
    // components: ComponentMap,
    component_incr: usize,
    component_types: HashMap<TypeId, usize>,
    entities: Vec<Entity>,
    incr_id: EntityId,
}


// System

pub trait System {
    fn update<'a>(&mut self, entities: impl Iterator<Item = &'a mut Entity>);
}

impl<'a> World {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn register<T: 'static>(&mut self) {
        self.component_types.insert(TypeId::of::<T>(), self.component_incr);
        self.component_incr += 1;
    }


    fn new_entity(&mut self) -> Entity {
        let e = Entity {
            id: self.incr_id,
            component_types: HashSet::new(),
            components: HashMap::new()
        };
        self.incr_id += 1;
        e
    }

    pub fn create_entity(&mut self) -> EntityBuilder {
        let entity = self.new_entity();
        EntityBuilder {
            world: self,
            entity,
        }
    }

    pub fn run_system<'b, S>(&'b mut self, system: &mut S, components: &[TypeId])
    where S: System {
        let set = HashSet::from_iter(components.iter().cloned());
        let i = self.entities.iter_mut();
        let matches = i.filter(|entity| {
            entity.component_types.is_superset(&set)
        });
        system.update(matches)
    }
}