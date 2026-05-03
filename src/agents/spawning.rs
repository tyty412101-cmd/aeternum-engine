use bevy::prelude::*;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

use super::*;
use crate::traits::TraitSet;
use crate::world::{WorldGrid, TileState, BiomeType};
use crate::SimConfig;

static HUMAN_NAMES: &[&str] = &[
    "Aldric", "Brynn", "Cedric", "Dara", "Edric", "Fiona", "Gareth", "Helena",
    "Ivan", "Janna", "Karl", "Lyra", "Marcus", "Nara", "Orin", "Petra",
    "Quinn", "Rhea", "Soren", "Thea", "Ulric", "Vera", "Wynn", "Xara",
];

static ELF_NAMES: &[&str] = &[
    "Aelindra", "Brinael", "Caelith", "Daerwyn", "Elowen", "Faelar", "Galanis",
    "Halindra", "Isilme", "Jarael", "Kaelis", "Lythien", "Miravel", "Naelori",
    "Orenthil", "Pyralei", "Quenara", "Rynael", "Sylvari", "Thalion",
];

static ORC_NAMES: &[&str] = &[
    "Azgrak", "Burz", "Cruush", "Durgat", "Ezgoth", "Funghar", "Grishnak",
    "Hulgat", "Ignash", "Juruk", "Kroshk", "Lugdush", "Muzgash", "Narzug",
    "Orgoth", "Pruzag", "Rukhash", "Skarug", "Turgon", "Uzgash",
];

static DWARF_NAMES: &[&str] = &[
    "Balin", "Craggor", "Dwalin", "Edrin", "Fundin", "Grimm", "Hjaldur",
    "Ingar", "Jorik", "Korgan", "Logrim", "Morin", "Norgar", "Orik",
    "Poldur", "Ragnar", "Stonehelm", "Thrain", "Ulfric", "Vargrim",
];

pub fn spawn_initial_populations(
    mut commands: Commands,
    config: Res<SimConfig>,
    world_grid: Res<WorldGrid>,
    tiles: Query<&TileState>,
) {
    let mut rng = ChaCha8Rng::seed_from_u64(config.seed + 42);

    let passable_tiles: Vec<(i32, i32)> = (0..world_grid.height)
        .flat_map(|y| (0..world_grid.width).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            if let Some(entity) = world_grid.get_tile(x, y) {
                if let Ok(tile) = tiles.get(entity) {
                    return tile.biome.is_passable();
                }
            }
            false
        })
        .collect();

    if passable_tiles.is_empty() {
        return;
    }

    let species_list = [Species::Human, Species::Elf, Species::Orc, Species::Dwarf];

    for species in &species_list {
        let start_idx = rng.gen_range(0..passable_tiles.len());
        let center = passable_tiles[start_idx];
        let center_world = world_grid.grid_to_world(center.0, center.1);

        for i in 0..config.initial_population_per_species {
            let offset_x = rng.gen_range(-3.0..3.0) * world_grid.tile_size;
            let offset_y = rng.gen_range(-3.0..3.0) * world_grid.tile_size;

            let pos = Vec2::new(
                center_world.x + offset_x,
                center_world.y + offset_y,
            );

            let gx = (pos.x / world_grid.tile_size).floor() as i32;
            let gy = (pos.y / world_grid.tile_size).floor() as i32;
            if gx < 0 || gx >= world_grid.width || gy < 0 || gy >= world_grid.height {
                continue;
            }

            if let Some(tile_entity) = world_grid.get_tile(gx, gy) {
                if let Ok(tile) = tiles.get(tile_entity) {
                    if !tile.biome.is_passable() {
                        continue;
                    }
                }
            }

            spawn_agent(&mut commands, &mut rng, *species, pos, &config);
        }
    }
}

pub fn spawn_agent(
    commands: &mut Commands,
    rng: &mut impl Rng,
    species: Species,
    pos: Vec2,
    config: &SimConfig,
) -> Entity {
    let base = species.base_stats();
    let names = match species {
        Species::Human => HUMAN_NAMES,
        Species::Elf => ELF_NAMES,
        Species::Orc => ORC_NAMES,
        Species::Dwarf => DWARF_NAMES,
    };
    let name = names[rng.gen_range(0..names.len())].to_string();

    let trait_set = TraitSet::generate_random(rng, 3);
    let size = if trait_set.has(crate::traits::Trait::Giant) {
        config.tile_size * 0.6
    } else if trait_set.has(crate::traits::Trait::Tiny) {
        config.tile_size * 0.2
    } else {
        config.tile_size * 0.35
    };

    let speed = base.speed * trait_set.speed_modifier();
    let strength = base.strength * trait_set.damage_modifier();

    commands.spawn((
        Sprite {
            color: species.color(),
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
        Agent { species, name },
        BiologicalState {
            health: base.max_health,
            max_health: base.max_health,
            health_regen: 0.1,
            stamina: 1.0,
            hunger: 0.0,
            age: 0,
            max_age: base.max_age,
            reproduction_cooldown: 0,
        },
        CognitiveState {
            threat_level: 0.0,
            opportunity_score: 0.0,
            risk_tolerance: base.aggression,
            decision_latency: (10.0 / base.intelligence).max(1.0) as u8,
            focus_target: None,
            current_action: AgentAction::Idle,
        },
        AgentStats {
            strength,
            speed,
            intelligence: base.intelligence,
            aggression: base.aggression + trait_set.aggression_modifier(),
            perception_radius: base.perception_radius,
            kill_count: 0,
            children_count: 0,
        },
        Allegiance {
            kingdom: None,
            settlement: None,
            loyalty: 0.5,
            role: AgentRole::Villager,
        },
        trait_set,
        AgentMarker,
    )).id()
}
