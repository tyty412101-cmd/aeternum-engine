use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use rand::Rng;

use crate::agents::*;
use crate::traits::TraitSet;
use crate::world::{WorldGrid, TileState};
use crate::SimTick;

pub struct CivilizationPlugin;

impl Plugin for CivilizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (
            settlement_formation_system,
            settlement_growth_system,
            kingdom_formation_system,
            diplomacy_system,
        ).chain().after(crate::agents::ai::agent_action_system));
    }
}

#[derive(Component, Debug)]
pub struct Settlement {
    pub name: String,
    pub species: Species,
    pub founded_tick: u64,
    pub population: u32,
    pub food_stockpile: f32,
    pub material_stockpile: f32,
    pub territory_radius: f32,
}

#[derive(Component)]
pub struct SettlementMarker;

#[derive(Component, Debug)]
pub struct Kingdom {
    pub name: String,
    pub species: Species,
    pub color: Color,
    pub founded_tick: u64,
    pub settlements: Vec<Entity>,
}

#[derive(Component)]
pub struct KingdomMarker;

#[derive(Component, Debug)]
pub struct DiplomacyState {
    pub relations: HashMap<Entity, DiplomaticRelation>,
}

#[derive(Debug, Clone)]
pub struct DiplomaticRelation {
    pub tension: f32,
    pub trust: f32,
    pub status: DiplomaticStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiplomaticStatus {
    Peace,
    Tense,
    War,
    Alliance,
}

impl DiplomaticStatus {
    pub fn display(&self) -> &'static str {
        match self {
            DiplomaticStatus::Peace => "Peace",
            DiplomaticStatus::Tense => "Tense",
            DiplomaticStatus::War => "WAR",
            DiplomaticStatus::Alliance => "Alliance",
        }
    }
}

static SETTLEMENT_NAMES: &[&str] = &[
    "Ironhold", "Brightwater", "Shadowfen", "Goldreach", "Stormvale",
    "Thornwall", "Deepstone", "Windbreak", "Frostmere", "Ashburn",
    "Silverpeak", "Ravenhollow", "Embercrest", "Moonridge", "Duskwood",
    "Oakenhaven", "Sunspire", "Grimwatch", "Starfall", "Blackrock",
    "Greendale", "Redmoor", "Bluehaven", "Whitecliff", "Darkvale",
    "Highgate", "Lowbrook", "Eastmarch", "Westford", "Northwatch",
];

static KINGDOM_NAMES: &[&str] = &[
    "The Iron Dominion", "Sylvan Realm", "The Bloodfang Horde", "Stonehelm Empire",
    "The Golden Republic", "Shadowveil Dynasty", "Thunder Kingdom", "The Emerald Court",
    "Ashborne Collective", "The Frost Sovereignty", "Sunfire Alliance", "Darkstone Pact",
    "The Silver Covenant", "Crimson Vanguard", "The Azure Throne", "Nightfall Dominion",
];

pub fn settlement_formation_system(
    mut commands: Commands,
    agents: Query<(Entity, &Transform, &Agent, &Allegiance), With<AgentMarker>>,
    settlements: Query<(Entity, &Transform, &Settlement), With<SettlementMarker>>,
    world_grid: Res<WorldGrid>,
    tiles: Query<&TileState>,
    tick: Res<SimTick>,
) {
    if tick.0 % 100 != 0 {
        return;
    }

    // Find clusters of unaffiliated agents
    let homeless: Vec<(Entity, Vec2, Species)> = agents
        .iter()
        .filter(|(_, _, _, a)| a.settlement.is_none())
        .map(|(e, t, a, _)| (e, t.translation.truncate(), a.species))
        .collect();

    if homeless.is_empty() {
        return;
    }

    // Group by species and proximity
    let mut clusters: HashMap<Species, Vec<Vec<(Entity, Vec2)>>> = HashMap::new();

    for (entity, pos, species) in &homeless {
        let cluster_list = clusters.entry(*species).or_default();

        let mut found_cluster = false;
        for cluster in cluster_list.iter_mut() {
            if let Some((_, center)) = cluster.first() {
                if pos.distance(*center) < world_grid.tile_size * 5.0 {
                    cluster.push((*entity, *pos));
                    found_cluster = true;
                    break;
                }
            }
        }
        if !found_cluster {
            cluster_list.push(vec![(*entity, *pos)]);
        }
    }

    let mut rng = rand::thread_rng();

    for (species, species_clusters) in &clusters {
        for cluster in species_clusters {
            if cluster.len() < 5 {
                continue;
            }

            // Calculate center
            let center = cluster.iter().fold(Vec2::ZERO, |acc, (_, p)| acc + *p) / cluster.len() as f32;

            // Check no existing settlement nearby
            let too_close = settlements.iter().any(|(_, t, _)| {
                t.translation.truncate().distance(center) < world_grid.tile_size * 10.0
            });
            if too_close {
                continue;
            }

            // Check tile suitability
            let (gx, gy) = world_grid.world_to_grid(center);
            if let Some(te) = world_grid.get_tile(gx, gy) {
                if let Ok(tile) = tiles.get(te) {
                    if !tile.biome.is_passable() || tile.fertility < 0.2 {
                        continue;
                    }
                }
            }

            let name_idx = rng.gen_range(0..SETTLEMENT_NAMES.len());
            let name = format!("{}", SETTLEMENT_NAMES[name_idx]);

            let settlement_entity = commands.spawn((
                Sprite {
                    color: Color::srgba(1.0, 0.85, 0.0, 0.6),
                    custom_size: Some(Vec2::splat(world_grid.tile_size * 2.0)),
                    ..default()
                },
                Transform::from_xyz(center.x, center.y, 0.5),
                Settlement {
                    name,
                    species: *species,
                    founded_tick: tick.0,
                    population: cluster.len() as u32,
                    food_stockpile: 50.0,
                    material_stockpile: 20.0,
                    territory_radius: world_grid.tile_size * 5.0,
                },
                SettlementMarker,
            )).id();

            break;
        }
    }
}

pub fn settlement_growth_system(
    mut settlements: Query<(&Transform, &mut Settlement), With<SettlementMarker>>,
    agents: Query<(&Transform, &Agent, &Allegiance), With<AgentMarker>>,
    world_grid: Res<WorldGrid>,
    tiles: Query<&TileState>,
    tick: Res<SimTick>,
) {
    if tick.0 % 50 != 0 {
        return;
    }

    for (stransform, mut settlement) in settlements.iter_mut() {
        let spos = stransform.translation.truncate();

        // Count agents near this settlement
        let nearby_count = agents
            .iter()
            .filter(|(t, a, _)| {
                a.species == settlement.species
                    && t.translation.truncate().distance(spos) < settlement.territory_radius
            })
            .count();

        settlement.population = nearby_count as u32;

        // Gather food from fertile tiles
        let (gx, gy) = world_grid.world_to_grid(spos);
        let radius = (settlement.territory_radius / world_grid.tile_size) as i32;
        let mut food_income = 0.0f32;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if let Some(te) = world_grid.get_tile(gx + dx, gy + dy) {
                    if let Ok(tile) = tiles.get(te) {
                        food_income += tile.fertility * 0.1;
                    }
                }
            }
        }

        settlement.food_stockpile += food_income;
        settlement.food_stockpile -= settlement.population as f32 * 0.5;
        settlement.food_stockpile = settlement.food_stockpile.max(0.0);

        // Grow territory with population
        if settlement.population > 10 {
            settlement.territory_radius = (settlement.territory_radius + 0.1).min(world_grid.tile_size * 15.0);
        }
    }
}

pub fn kingdom_formation_system(
    mut commands: Commands,
    settlements: Query<(Entity, &Transform, &Settlement), (With<SettlementMarker>, Without<KingdomMarker>)>,
    kingdoms: Query<(Entity, &Kingdom), With<KingdomMarker>>,
    tick: Res<SimTick>,
) {
    if tick.0 % 500 != 0 {
        return;
    }

    let mut rng = rand::thread_rng();

    // Group settlements by species and proximity
    let settlement_list: Vec<(Entity, Vec2, Species, u32)> = settlements
        .iter()
        .map(|(e, t, s)| (e, t.translation.truncate(), s.species, s.population))
        .collect();

    let mut used: HashSet<Entity> = HashSet::new();

    for (entity, pos, species, pop) in &settlement_list {
        if used.contains(entity) || *pop < 10 {
            continue;
        }

        // Find nearby same-species settlements
        let mut group = vec![*entity];
        for (other_e, other_pos, other_species, _) in &settlement_list {
            if other_e == entity || used.contains(other_e) || other_species != species {
                continue;
            }
            if pos.distance(*other_pos) < 500.0 {
                group.push(*other_e);
            }
        }

        if group.len() >= 2 {
            let name_idx = rng.gen_range(0..KINGDOM_NAMES.len());

            let kingdom_entity = commands.spawn((
                Kingdom {
                    name: KINGDOM_NAMES[name_idx].to_string(),
                    species: *species,
                    color: Color::srgba(
                        rng.gen_range(0.3..1.0),
                        rng.gen_range(0.3..1.0),
                        rng.gen_range(0.3..1.0),
                        0.3,
                    ),
                    founded_tick: tick.0,
                    settlements: group.clone(),
                },
                KingdomMarker,
                DiplomacyState {
                    relations: HashMap::new(),
                },
            )).id();

            for e in &group {
                used.insert(*e);
            }
        }
    }
}

pub fn diplomacy_system(
    mut kingdoms: Query<(Entity, &Kingdom, &mut DiplomacyState), With<KingdomMarker>>,
    tick: Res<SimTick>,
) {
    if tick.0 % 200 != 0 {
        return;
    }

    let kingdom_data: Vec<(Entity, Species, usize)> = kingdoms
        .iter()
        .map(|(e, k, _)| (e, k.species, k.settlements.len()))
        .collect();

    for (entity, _kingdom, mut diplo) in kingdoms.iter_mut() {
        for (other_entity, other_species, other_size) in &kingdom_data {
            if *other_entity == entity {
                continue;
            }

            let relation = diplo.relations.entry(*other_entity).or_insert(DiplomaticRelation {
                tension: 0.2,
                trust: 0.3,
                status: DiplomaticStatus::Peace,
            });

            // Species affect base tension
            relation.tension += 0.01;

            // Power imbalance increases tension
            relation.tension += 0.005;

            relation.tension = relation.tension.clamp(0.0, 1.5);
            relation.trust = relation.trust.clamp(0.0, 1.0);

            // Update status
            if relation.tension > 0.8 && relation.status != DiplomaticStatus::War {
                relation.status = DiplomaticStatus::War;
            } else if relation.tension > 0.5 && relation.status == DiplomaticStatus::Peace {
                relation.status = DiplomaticStatus::Tense;
            } else if relation.tension < 0.3 && relation.trust > 0.6 {
                relation.status = DiplomaticStatus::Alliance;
            }
        }
    }
}
