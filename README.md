# SuperInstance Core — ECS Entity Management

The foundational **Entity Component System (ECS)** for SuperInstance — providing generational entity IDs, type-erased component storage, and query iteration over the fleet's data model.

## Why It Matters

An ECS (Entity-Component-System) is the dominant architecture for simulation-heavy applications: every game engine (Unity DOTS, Bevy, Unreal Mass), physics simulator, and now fleet management system uses it. The pattern separates **identity** (entities are just IDs) from **data** (components are plain structs stored in contiguous arrays) from **logic** (systems iterate over entities with specific component combinations). This enables cache-friendly iteration, data-oriented design, and flexible composition — adding a component to an entity is O(1), and querying "all entities with Position + Velocity" is a single hash lookup plus array scan. SuperInstance Core provides this foundation with generational indices for safe entity lifecycle management.

## How It Works

### Entity = ID + Generation

```
Entity {
    id: u64,          // unique sequential ID
    generation: u32,  // bumped on each despawn
}
```

The generation counter prevents the ABA problem: after entity `id=5, gen=0` is despawned and its slot reused, old references holding `gen=0` are safely rejected.

### Component Storage: Type-Erased Columns

Each component type `T: Component` gets its own `TypedStorage<T>`:

```
TypedStorage<T> {
    data: HashMap<EntityId, T>,
}
```

Storages are registered in a `HashMap<TypeId, Box<dyn ComponentStorage>>` — type-erased so the World can hold heterogeneous columns.

### World — The Central Registry

```
World {
    next_id: EntityId,
    entities: HashMap<EntityId, u32>,  // id → generation
    storages: HashMap<TypeId, Box<dyn ComponentStorage>>,
    pending_delete: Vec<EntityId>,
}
```

### Lifecycle

```
spawn() → Entity                         // allocate new ID + generation 0
add_component(entity, T)                  // insert into T's storage
get_component::<T>(entity) → Option<&T>   // type-directed lookup
despawn(entity)                           → pending_delete
flush()                                    // process deletions: bump generations, remove components
```

Deferred deletion (`despawn` + `flush`) enables batch processing — mark multiple entities for deletion, then clean up once.

### Query

```
Query::<Position>::new(world)
    .iter() → (EntityId, &Position)
```

Returns all entities that have the queried component type. Multi-component queries would join across storages.

### Complexity

| Operation | Time |
|-----------|------|
| spawn | O(1) |
| add_component | O(1) amortized |
| get_component | O(1) |
| despawn | O(1) |
| flush (delete k entities) | O(k × c) where c = component types |
| Query iter | O(n) over component storage |

### Compared to Alternatives

| Architecture | Cache locality | Flexibility | Complexity |
|-------------|---------------|-------------|------------|
| ECS (this crate, HashMap-based) | Poor (HashMap) | Excellent | Moderate |
| Archetypal ECS (Bevy) | Excellent (SoA) | Excellent | High |
| OOP inheritance | N/A | Poor | Low |

This crate uses HashMap-based storage for simplicity. Production ECS frameworks use archetype-based storage (grouping entities with identical component sets into contiguous arrays) for cache-line-friendly iteration.

## Quick Start

```rust
use superinstance_core::{World, Entity};

#[derive(Debug, PartialEq)]
struct Position { x: f64, y: f64 }

#[derive(Debug, PartialEq)]
struct Health(i32);

fn main() {
    let mut world = World::new();

    // Spawn entities
    let e1 = world.spawn();
    world.add_component(e1, Position { x: 0.0, y: 0.0 });
    world.add_component(e1, Health(100));

    let e2 = world.spawn();
    world.add_component(e2, Position { x: 5.0, y: 3.0 });

    // Query
    let query = World::storage::<Position>(&world).unwrap();
    for (id, pos) in query.iter() {
        println!("Entity {:?} at ({}, {})", id, pos.x, pos.y);
    }

    // Despawn
    world.despawn(e1);
    world.flush();
    assert!(!world.is_alive(e1));
}
```

## API

### `World`
- `new()` / `spawn() -> Entity`
- `add_component<T>(entity, component)`
- `get_component<T>(entity) -> Option<&T>` / `get_component_mut<T>(entity) -> Option<&mut T>`
- `is_alive(entity) -> bool`
- `despawn(entity)` / `flush()`
- `entity_count()` / `storage::<T>()`

### `Entity`
- `id: EntityId (u64)`, `generation: u32`

### `Query<'a, T>`
- `new(world) -> Option<Self>` — get iterator over component T
- `iter() -> impl Iterator<Item = (EntityId, &T)>`

### `Component` trait
- Blanket-implemented for `T: Any + Send + Sync`

## Architecture Notes

SuperInstance Core is the foundation of fleet entity management. Each ship is an Entity; its conservation state (γ, η, C) is a component; the Cocapn's routing logic is a system that queries entities with specific component combinations. The γ + η = C conservation law is enforced as a system invariant across the ECS world. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Nystrom, R. (2014). *Game Programming Patterns*, "Component" chapter.
- Bevy Engine. *ECS Architecture*. bevyengine.org/learn
- Doerr, B. (2019). *Data-Oriented Design*. blog storage docs.

## License

MIT
