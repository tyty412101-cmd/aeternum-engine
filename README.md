# Aeternum Engine

**Simulated Chaos, Bound by Rules**

A multi-scale emergent civilization simulation built with Rust and Bevy ECS. Every war, alliance, collapse, and golden age is the result of systems colliding in real time -- nothing is scripted.

![Rust](https://img.shields.io/badge/Rust-1.83+-orange.svg)
![Bevy](https://img.shields.io/badge/Bevy-0.15-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

## What It Is

Aeternum Engine is a god-game simulation where you observe and influence a world of autonomous agents. Four species -- **Humans**, **Elves**, **Orcs**, and **Dwarves** -- live, fight, build, and die according to their own AI-driven decisions.

### Core Systems

- **Autonomous Agent AI** -- Each entity has its own decision loop: perceive, evaluate, decide, act. Agents seek food, fight enemies, flee danger, explore, and socialize based on weighted scoring.
- **Trait System** -- 46 traits across 6 categories (Combat, Cognitive, Biological, Social, Environmental, Special). Traits combine non-linearly to create emergent archetypes. Traits are inherited and mutate across generations.
- **Civilization Emergence** -- Agents naturally cluster into settlements, which grow into kingdoms with territory, diplomacy, and leadership.
- **Environmental Simulation** -- Tile-based world with biome-dependent properties. Heat, corruption, and fire propagate via diffusion. Terrain affects movement, resources, and survival.
- **Conflict & Diplomacy** -- Kingdoms accumulate tension based on border overlap, resource competition, and power imbalance. Wars erupt when tension exceeds thresholds. Combat is resolved at the individual level.
- **God Powers** -- Reshape terrain, paint biomes, spawn entities, assign traits, and unleash disasters (meteors, lightning, fire, plague).
- **History Recording** -- The simulation records kingdom formation, wars, and territorial evolution. Every world creates its own unique history.

### Species

| Species | Strengths | Traits |
|---------|-----------|--------|
| **Human** | Balanced stats, adaptable | Moderate in all areas |
| **Elf** | Long-lived, intelligent, perceptive | Strategic, low fertility |
| **Orc** | Strong, aggressive, high fertility | Combat-focused, short-lived |
| **Dwarf** | Tough, loyal, long-lived | Defensive, slow but resilient |

## Getting Started

### Prerequisites

- **Rust** 1.83+ (install via [rustup](https://rustup.rs/))
- **Linux system packages** (for Bevy):
  ```bash
  sudo apt install pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
  ```

### Build & Run

```bash
git clone <repo-url>
cd aeternum_engine
cargo run
```

First build takes a while (Bevy has many dependencies). Subsequent builds are fast due to incremental compilation.

### Controls

| Key | Action |
|-----|--------|
| **WASD / Arrow Keys** | Pan camera |
| **Mouse Scroll** | Zoom in/out |
| **Left Click** | Select unit/tile or use god tool |
| **Space** | Pause/Resume |
| **1** | Speed: 0.5x |
| **2** | Speed: 1x |
| **3** | Speed: 3x |
| **4** | Speed: 10x |

### God Powers (Left Panel)

- **Terrain**: Raise/Lower land
- **Biomes**: Paint Grass, Forest, Desert, Water
- **Life**: Spawn Human, Elf, Orc, Dwarf
- **Disasters**: Meteor, Lightning, Fire, Plague
- **Select**: Click units/tiles to inspect them

## Architecture

```
src/
  main.rs              -- App setup, camera, time controls
  world/
    mod.rs             -- Tile types, world grid, biome system
    generation.rs      -- Procedural world generation (Perlin noise)
    environment.rs     -- Environmental diffusion (heat, corruption, fire)
  agents/
    mod.rs             -- Agent components, species, biological state
    ai.rs              -- Decision engine (perceive, evaluate, decide, act)
    spawning.rs        -- Population initialization
  traits/
    mod.rs             -- 46 traits, modifiers, inheritance, mutation
  civilization/
    mod.rs             -- Settlements, kingdoms, diplomacy
  combat/
    mod.rs             -- Individual-level combat resolution
  god_powers/
    mod.rs             -- Player tools, terrain editing, disasters
  ui/
    mod.rs             -- egui panels (top bar, tools, inspection)
```

### ECS Design

Everything is an entity with components:

| Entity | Key Components |
|--------|---------------|
| Agent | Species, BiologicalState, CognitiveState, TraitSet, Allegiance |
| Tile | TileState (biome, elevation, temperature, moisture, fertility) |
| Settlement | Population, resources, territory |
| Kingdom | Diplomacy, settlements, leadership |

Systems run in order: Environment -> Agent AI -> Civilization -> Combat -> Rendering

## Configuration

Default simulation parameters (in `SimConfig`):

| Parameter | Default | Description |
|-----------|---------|-------------|
| `world_width` | 128 | Map width in tiles |
| `world_height` | 128 | Map height in tiles |
| `tile_size` | 8.0 | Pixel size per tile |
| `seed` | 42 | World generation seed |
| `initial_population_per_species` | 30 | Starting population per species |

## Roadmap

- [ ] Army formations and siege mechanics
- [ ] Advanced building construction
- [ ] Trade routes between settlements
- [ ] Technology trees
- [ ] Fauna ecosystem (predators/prey)
- [ ] History timeline UI
- [ ] Save/Load system
- [ ] Sound and music
- [ ] Trait engineering UI
- [ ] WASM web build

## License

MIT
