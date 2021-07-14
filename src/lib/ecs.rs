use crate::HashMap;
use core::any::Any;
use std::any::TypeId;

type SystemId = TypeId;
type EntityId = u64;
type ComponentMap = HashMap<SystemId, Box<dyn Component>>;

trait Component: Any {}

struct EntityBuilder<'a> {
    world: &'a mut World,
    comps: ComponentMap,
}

impl<'a> EntityBuilder<'a> {
    fn with<C: Component>(&mut self, component: C) {
        self.comps.insert(TypeId::of::<C>(), Box::new(component));
    }
    fn build(self) {
        let entity = Entity {
            id: self.world.incr_id, // TODO,
            components: self.comps,
        };
        self.world.incr_id += 1;
        self.world.entities.push(entity);
    }
}

struct Entities {}

struct Entity {
    id: EntityId,
    components: ComponentMap,
}

// World is a base container class that we can register components and entities to.
pub struct World {
    components: ComponentMap,
    entities: Vec<Entity>,
    incr_id: EntityId,
}

trait System {
    type SystemData;
    fn update(&mut self, entities: Entities) {}
}

impl World {
    fn register<T: Component>(&mut self) {}
    fn create_entity(&mut self) -> EntityBuilder {
        EntityBuilder {
            world: self,
            comps: ComponentMap::new(),
        }
    }
    fn run_system<S: System>(&mut self, system: S) {}
}
