use bevy::prelude::*;
use rand::Rng;

use super::*;
use crate::traits::{Trait, TraitSet};
use crate::world::{WorldGrid, TileState};
use crate::SimTick;

pub fn agent_perception_system(
    mut agents: Query<(
        &Transform,
        &AgentStats,
        &mut CognitiveState,
        &Agent,
        &TraitSet,
    ), With<AgentMarker>>,
    others: Query<(&Transform, &Agent, &BiologicalState), With<AgentMarker>>,
    world_grid: Res<WorldGrid>,
    tiles: Query<&TileState>,
) {
    let positions: Vec<(Entity, Vec3, Species)> = others
        .iter()
        .map(|(t, a, _)| (Entity::PLACEHOLDER, t.translation, a.species))
        .collect();

    for (transform, stats, mut cognitive, agent, traits) in agents.iter_mut() {
        let pos = transform.translation.truncate();
        let radius = stats.perception_radius * world_grid.tile_size;

        let mut threat = 0.0f32;
        let mut opportunity = 0.0f32;
        let mut nearby_enemies = 0;
        let mut nearby_allies = 0;

        for (_, other_pos, other_species) in &positions {
            let dist = pos.distance(other_pos.truncate());
            if dist > radius || dist < 1.0 {
                continue;
            }
            if *other_species != agent.species {
                nearby_enemies += 1;
                threat += (1.0 - dist / radius) * 0.3;
            } else {
                nearby_allies += 1;
                opportunity += 0.1;
            }
        }

        let (gx, gy) = world_grid.world_to_grid(pos);
        if let Some(tile_entity) = world_grid.get_tile(gx, gy) {
            if let Ok(tile) = tiles.get(tile_entity) {
                if tile.on_fire {
                    threat += 0.5;
                }
                if tile.corruption > 0.3 {
                    threat += tile.corruption * 0.3;
                }
                if tile.fertility > 0.5 {
                    opportunity += tile.fertility * 0.2;
                }
            }
        }

        if traits.has(Trait::Paranoid) {
            threat *= 1.5;
        }
        if traits.has(Trait::Brave) {
            threat *= 0.6;
        }

        cognitive.threat_level = threat.min(1.0);
        cognitive.opportunity_score = opportunity.min(1.0);
    }
}

pub fn agent_decision_system(
    mut agents: Query<(
        Entity,
        &BiologicalState,
        &AgentStats,
        &mut CognitiveState,
        &TraitSet,
        &Allegiance,
    ), With<AgentMarker>>,
    tick: Res<SimTick>,
) {
    for (entity, bio, stats, mut cognitive, traits, allegiance) in agents.iter_mut() {
        if tick.0 % cognitive.decision_latency as u64 != 0 {
            continue;
        }

        let mut scores = Vec::with_capacity(8);

        // Eat score - based on hunger
        let eat_score = bio.hunger * 100.0
            * if traits.has(Trait::Impulsive) { 1.3 } else { 1.0 };
        scores.push((AgentAction::SeekFood, eat_score));

        // Rest score - based on stamina and health
        let rest_score = (1.0 - bio.stamina) * 70.0 + (1.0 - bio.health / bio.max_health) * 50.0;
        scores.push((AgentAction::Rest, rest_score));

        // Flee score - based on threat
        let flee_score = cognitive.threat_level * (1.0 - cognitive.risk_tolerance) * 80.0
            * if traits.has(Trait::Cowardly) { 1.5 } else { 1.0 }
            * if traits.has(Trait::Brave) { 0.3 } else { 1.0 };
        scores.push((AgentAction::Flee, flee_score));

        // Fight score - based on aggression and threat
        let fight_score = (stats.aggression * 60.0 + cognitive.threat_level * 20.0)
            * if traits.has(Trait::Bloodthirsty) { 1.5 } else { 1.0 }
            * if traits.has(Trait::Berserker) {
                1.0 + (1.0 - bio.health / bio.max_health)
            } else {
                1.0
            };
        scores.push((AgentAction::Fight, fight_score));

        // Work score
        let work_score = 40.0 * (1.0 - bio.hunger) * (1.0 - cognitive.threat_level)
            * if allegiance.settlement.is_some() { 1.5 } else { 0.5 };
        scores.push((AgentAction::Work, work_score));

        // Explore score
        let explore_score = 25.0
            * if traits.has(Trait::Curious) { 2.0 } else { 1.0 }
            * (1.0 - bio.hunger)
            * (1.0 - cognitive.threat_level);
        scores.push((AgentAction::Explore, explore_score));

        // Wander score (default fallback)
        scores.push((AgentAction::Wander, 15.0));

        // Add random variance
        let variance = ((entity.index() as u64 ^ tick.0) % 100) as f32 / 100.0 * 10.0 - 5.0;
        for (_, score) in &mut scores {
            *score += variance;
        }

        // Pick highest
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        cognitive.current_action = scores[0].0;
    }
}

pub fn agent_action_system(
    mut agents: Query<(
        &mut Transform,
        &CognitiveState,
        &AgentStats,
        &mut BiologicalState,
        &TraitSet,
    ), With<AgentMarker>>,
    world_grid: Res<WorldGrid>,
    tiles: Query<&TileState>,
    tick: Res<SimTick>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut transform, cognitive, stats, mut bio, traits) in agents.iter_mut() {
        let pos = transform.translation.truncate();
        let base_speed = stats.speed * world_grid.tile_size * 0.3;

        let (gx, gy) = world_grid.world_to_grid(pos);
        let movement_cost = if let Some(te) = world_grid.get_tile(gx, gy) {
            tiles.get(te).map(|t| t.biome.movement_cost()).unwrap_or(1.0)
        } else {
            1.0
        };

        let effective_speed = base_speed / movement_cost * dt;

        match cognitive.current_action {
            AgentAction::Wander | AgentAction::Explore => {
                let angle = ((tick.0 as f32 * 0.1) + (transform.translation.x * 0.01) + (transform.translation.y * 0.013)).sin() * std::f32::consts::TAU;
                let dir = Vec2::new(angle.cos(), angle.sin());
                let new_pos = pos + dir * effective_speed;

                let (ngx, ngy) = world_grid.world_to_grid(new_pos);
                if ngx >= 0 && ngx < world_grid.width && ngy >= 0 && ngy < world_grid.height {
                    if let Some(te) = world_grid.get_tile(ngx, ngy) {
                        if let Ok(tile) = tiles.get(te) {
                            if tile.biome.is_passable() {
                                transform.translation.x = new_pos.x;
                                transform.translation.y = new_pos.y;
                            }
                        }
                    }
                }

                bio.stamina = (bio.stamina - 0.001 * dt).max(0.0);
                bio.hunger = (bio.hunger + 0.002 * dt).min(1.0);
            }
            AgentAction::SeekFood => {
                // Move toward higher fertility tiles
                let mut best_dir = Vec2::ZERO;
                let mut best_fertility = 0.0f32;

                for dy in -2i32..=2 {
                    for dx in -2i32..=2 {
                        if dx == 0 && dy == 0 { continue; }
                        let nx = gx + dx;
                        let ny = gy + dy;
                        if let Some(te) = world_grid.get_tile(nx, ny) {
                            if let Ok(tile) = tiles.get(te) {
                                if tile.fertility > best_fertility && tile.biome.is_passable() {
                                    best_fertility = tile.fertility;
                                    let target = world_grid.grid_to_world(nx, ny);
                                    best_dir = (target - pos).normalize_or_zero();
                                }
                            }
                        }
                    }
                }

                if best_dir != Vec2::ZERO {
                    let new_pos = pos + best_dir * effective_speed;
                    transform.translation.x = new_pos.x;
                    transform.translation.y = new_pos.y;
                }

                // Eat if on fertile tile
                if let Some(te) = world_grid.get_tile(gx, gy) {
                    if let Ok(tile) = tiles.get(te) {
                        if tile.fertility > 0.3 {
                            bio.hunger = (bio.hunger - 0.05 * dt).max(0.0);
                            bio.health = (bio.health + 0.5 * dt).min(bio.max_health);
                        }
                    }
                }

                bio.stamina = (bio.stamina - 0.002 * dt).max(0.0);
            }
            AgentAction::Rest => {
                bio.stamina = (bio.stamina + 0.01 * dt).min(1.0);
                bio.health = (bio.health + bio.health_regen * dt).min(bio.max_health);
                if traits.has(Trait::Regenerative) {
                    bio.health = (bio.health + bio.health_regen * 2.0 * dt).min(bio.max_health);
                }
                bio.hunger = (bio.hunger + 0.001 * dt).min(1.0);
            }
            AgentAction::Flee => {
                // Move away from center (simple flee behavior)
                let flee_dir = Vec2::new(
                    if pos.x > world_grid.width as f32 * world_grid.tile_size * 0.5 { 1.0 } else { -1.0 },
                    if pos.y > world_grid.height as f32 * world_grid.tile_size * 0.5 { 1.0 } else { -1.0 },
                ).normalize_or_zero();

                let new_pos = pos + flee_dir * effective_speed * 1.5;
                let (ngx, ngy) = world_grid.world_to_grid(new_pos);
                if ngx >= 0 && ngx < world_grid.width && ngy >= 0 && ngy < world_grid.height {
                    if let Some(te) = world_grid.get_tile(ngx, ngy) {
                        if let Ok(tile) = tiles.get(te) {
                            if tile.biome.is_passable() {
                                transform.translation.x = new_pos.x;
                                transform.translation.y = new_pos.y;
                            }
                        }
                    }
                }
                bio.stamina = (bio.stamina - 0.005 * dt).max(0.0);
            }
            AgentAction::Fight => {
                bio.stamina = (bio.stamina - 0.003 * dt).max(0.0);
                bio.hunger = (bio.hunger + 0.003 * dt).min(1.0);
            }
            AgentAction::Work => {
                bio.stamina = (bio.stamina - 0.002 * dt).max(0.0);
                bio.hunger = (bio.hunger + 0.002 * dt).min(1.0);
            }
            _ => {
                bio.hunger = (bio.hunger + 0.001 * dt).min(1.0);
            }
        }
    }
}

pub fn agent_aging_system(
    mut commands: Commands,
    mut agents: Query<(Entity, &mut BiologicalState, &TraitSet), With<AgentMarker>>,
    tick: Res<SimTick>,
) {
    if tick.0 % 10 != 0 {
        return;
    }

    for (entity, mut bio, traits) in agents.iter_mut() {
        bio.age += 1;

        // Aging effects
        if bio.age > bio.max_age && !traits.has(Trait::Immortal) {
            // Death by old age
            commands.entity(entity).despawn();
            continue;
        }

        // Starvation
        if bio.hunger >= 1.0 {
            bio.health -= 1.0;
            if bio.health <= 0.0 {
                commands.entity(entity).despawn();
                continue;
            }
        }

        // Short-lived accelerated aging
        if traits.has(Trait::ShortLived) {
            bio.age += 1;
        }

        // Reproduction cooldown
        if bio.reproduction_cooldown > 0 {
            bio.reproduction_cooldown -= 1;
        }
    }
}

pub fn agent_hunger_system(
    mut agents: Query<&mut BiologicalState, With<AgentMarker>>,
    tick: Res<SimTick>,
) {
    if tick.0 % 20 != 0 {
        return;
    }

    for mut bio in agents.iter_mut() {
        bio.hunger = (bio.hunger + 0.005).min(1.0);
    }
}
