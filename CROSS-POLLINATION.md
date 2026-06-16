# CROSS-POLLINATION.md — superinstance-core

> **Conservation Law Connection:** ECS substrate manages γ/η flow

## Role in the Conservation Law

`superinstance-core` provides the Entity Component System (ECS) that all fleet
components run on. In the conservation law framework:

- **Entities** = agents (each has a γ/η budget)
- **Components** = γ and η contributions (stored as typed data)
- **Systems** = functions that transform γ inputs into γ outputs (with η overhead)
- **Queries** = the routing layer that connects γ producers to γ consumers

The ECS is the **physical substrate** on which γ and η flow. Conservation law
violations at the ECS level (entity leaks, query inefficiency, component bloat)
directly inflate η across the entire fleet.

## delta-clt Verification Results

The delta-clt dependency graph simulation (Section 5) models exactly the ECS pattern:
- Nodes = entities
- Edges = component queries (routing)
- γ = payload entities (productive data)
- η = query overhead + health checks + graph compilation

Results: at 50 entities with 30% edge density, η ≈ 3%. The ECS architecture
is conservation-law-efficient at scale. Generational entity IDs prevent memory
leaks (a major η source in naive systems).

## Cross-Repo Connections

### → ternary-fleet
Fleet sub-crates register their components with the ECS. `ternary-conv` registers
computation results, `ternary-fuse` registers merged data, `ternary-em` registers
inference outputs. All γ flows through the ECS.

**Shared:** Both are fleet infrastructure. Fleet components ARE ECS entities.
**Different:** `core` provides the entity system; `fleet` provides the applications.

### → ternary-search-rs
Search results registered as ECS entities enable fleet-wide discovery. A component
query can find "all entities matching pattern X" using the search index.

**Shared:** Both manage collections of typed entities.
**Different:** `core` is CRUD + iteration; `search-rs` is semantic similarity.

### → superinstance-protocol
Protocol messages are serialized ECS component data. The ECS defines the data
model; the protocol defines the wire format. Together: complete fleet data pipeline.

**Shared:** Both define how fleet data is structured.
**Different:** `core` is in-memory; `protocol` is wire-format.

## Fleet Position

```
┌────────────────────────────────────────────────────┐
│  superinstance-core — THE SUBSTRATE                 │
│                                                     │
│  ECS Pipeline:                                      │
│  Entity ──► Component ──► System ──► Query ──► γ   │
│                                       └──► η       │
│                                                     │
│  Generational IDs prevent entity leaks (η source)  │
│  Type-erased storage enables flexible γ routing    │
│  Query iteration is the γ distribution mechanism    │
│                                                     │
│  Carries:                                           │
│  ├─ ternary-fleet components (γ producers)          │
│  ├─ ternary-search-rs results (discovery γ)         │
│  └─ superinstance-protocol payloads (wire γ)        │
│                                                     │
│  η sources: query overhead, health checks,         │
│             graph compilation, memory management    │
│  η floor: δ(n_entities)                             │
└────────────────────────────────────────────────────┘
```

