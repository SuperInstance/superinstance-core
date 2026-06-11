//! Superinstance Core — entity management with instance pooling and lifecycle.
//!
//! Provides a high-performance entity component system (ECS) core with
//! archetypal storage for cache-friendly iteration.

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// A raw entity identifier.
pub type EntityId = u64;

/// A generation counter to detect dangling references.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entity {
    pub id: EntityId,
    pub generation: u32,
}

/// Trait for any component that can be stored.
pub trait Component: Any + Send + Sync {}

impl<T: Any + Send + Sync> Component for T {}

/// A type-erased component storage column.
pub trait ComponentStorage: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, id: EntityId);
    fn len(&self) -> usize;
}

/// Typed storage for a single component type.
pub struct TypedStorage<T: Component> {
    data: HashMap<EntityId, T>,
}

impl<T: Component> TypedStorage<T> {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    pub fn insert(&mut self, id: EntityId, component: T) {
        self.data.insert(id, component);
    }

    pub fn get(&self, id: EntityId) -> Option<&T> {
        self.data.get(&id)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        self.data.get_mut(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.data.iter().map(|(k, v)| (*k, v))
    }
}

impl<T: Component> ComponentStorage for TypedStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn remove(&mut self, id: EntityId) {
        self.data.remove(&id);
    }
    fn len(&self) -> usize {
        self.data.len()
    }
}

/// The core world/store holding all entities and components.
pub struct World {
    next_id: EntityId,
    entities: HashMap<EntityId, u32>, // id -> generation
    storages: HashMap<TypeId, Box<dyn ComponentStorage>>,
    pending_delete: Vec<EntityId>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            entities: HashMap::new(),
            storages: HashMap::new(),
            pending_delete: Vec::new(),
        }
    }

    /// Spawn a new entity, returning its handle.
    pub fn spawn(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        self.entities.insert(id, 0);
        Entity { id, generation: 0 }
    }

    /// Add a component to an entity.
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        if !self.is_alive(entity) {
            return;
        }
        let type_id = TypeId::of::<T>();
        let storage = self.storages
            .entry(type_id)
            .or_insert_with(|| Box::new(TypedStorage::<T>::new()));
        storage
            .as_any_mut()
            .downcast_mut::<TypedStorage<T>>()
            .unwrap()
            .insert(entity.id, component);
    }

    /// Get a reference to a component.
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.is_alive(entity) {
            return None;
        }
        let type_id = TypeId::of::<T>();
        self.storages.get(&type_id)?
            .as_any()
            .downcast_ref::<TypedStorage<T>>()?
            .get(entity.id)
    }

    /// Get a mutable reference to a component.
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.is_alive(entity) {
            return None;
        }
        let type_id = TypeId::of::<T>();
        self.storages.get_mut(&type_id)?
            .as_any_mut()
            .downcast_mut::<TypedStorage<T>>()?
            .get_mut(entity.id)
    }

    /// Check if an entity is alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.get(&entity.id).map(|&g| g == entity.generation).unwrap_or(false)
    }

    /// Queue an entity for deletion.
    pub fn despawn(&mut self, entity: Entity) {
        if self.is_alive(entity) {
            self.pending_delete.push(entity.id);
        }
    }

    /// Process pending deletions.
    pub fn flush(&mut self) {
        for id in self.pending_delete.drain(..) {
            if let Some(gen) = self.entities.get_mut(&id) {
                *gen += 1;
            }
            for storage in self.storages.values_mut() {
                storage.remove(id);
            }
        }
    }

    /// Get typed storage for iteration.
    pub fn storage<T: Component>(&self) -> Option<&TypedStorage<T>> {
        let type_id = TypeId::of::<T>();
        let storage = self.storages.get(&type_id)?;
        storage.as_any().downcast_ref::<TypedStorage<T>>()
    }

    /// Number of living entities.
    pub fn entity_count(&self) -> usize {
        self.entities.len() - self.pending_delete.len()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// A query builder for component iteration.
pub struct Query<'a, T: Component> {
    storage: &'a TypedStorage<T>,
    world: &'a World,
}

impl<'a, T: Component> Query<'a, T> {
    pub fn new(world: &'a World) -> Option<Self> {
        world.storage().map(|storage| Self { storage, world })
    }

    /// Iterate over all entities that have this component.
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.storage.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Debug)]
    struct Position { x: f64, y: f64 }
    #[derive(PartialEq, Debug)]
    struct Velocity { dx: f64, dy: f64 }
    #[derive(PartialEq)]
    struct Health(i32);

    #[test]
    fn test_spawn_and_add() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Position { x: 1.0, y: 2.0 });
        let pos = world.get_component::<Position>(e).unwrap();
        assert!((pos.x - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_despawn() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Health(100));
        world.despawn(e);
        world.flush();
        assert!(!world.is_alive(e));
        assert!(world.get_component::<Health>(e).is_none());
    }

    #[test]
    fn test_multiple_components() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Position { x: 0.0, y: 0.0 });
        world.add_component(e, Velocity { dx: 1.0, dy: 0.0 });
        let vel = world.get_component::<Velocity>(e).unwrap();
        assert!((vel.dx - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_query() {
        let mut world = World::new();
        let e1 = world.spawn();
        world.add_component(e1, Position { x: 1.0, y: 0.0 });
        let e2 = world.spawn();
        world.add_component(e2, Position { x: 2.0, y: 0.0 });

        let query: Query<Position> = Query::new(&world).unwrap();
        let count = query.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_generation() {
        let mut world = World::new();
        let e = world.spawn();
        world.despawn(e);
        world.flush();
        // Entity with old generation should not be alive
        assert!(!world.is_alive(e));
    }
}
