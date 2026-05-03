use bevy::prelude::*;

use crate::agents::*;
use crate::traits::{Trait, TraitSet};
use crate::world::WorldGrid;
use crate::SimTick;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, combat_system.after(crate::agents::ai::agent_action_system));
    }
}

fn combat_system(
    mut commands: Commands,
    mut agents: Query<(
        Entity,
        &Transform,
        &Agent,
        &mut BiologicalState,
        &mut AgentStats,
        &CognitiveState,
        &TraitSet,
    ), With<AgentMarker>>,
    world_grid: Res<WorldGrid>,
    tick: Res<SimTick>,
) {
    if tick.0 % 5 != 0 {
        return;
    }

    // Collect combatant data
    let combatants: Vec<(Entity, Vec2, Species, f32, f32, bool)> = agents
        .iter()
        .filter(|(_, _, _, _, _, c, _)| c.current_action == AgentAction::Fight)
        .map(|(e, t, a, bio, stats, _, traits)| {
            let damage = stats.strength * traits.damage_modifier();
            let defense = traits.defense_modifier();
            let is_berserker = traits.has(Trait::Berserker);
            (e, t.translation.truncate(), a.species, damage, defense, is_berserker)
        })
        .collect();

    let mut damage_list: Vec<(Entity, f32)> = Vec::new();
    let mut kill_credits: Vec<Entity> = Vec::new();

    for (entity, pos, species, damage, _, is_berserker) in &combatants {
        // Find nearest enemy in combat range
        let combat_range = world_grid.tile_size * 1.5;
        let mut nearest_enemy: Option<(Entity, f32)> = None;

        for (other_e, other_pos, other_species, _, _, _) in &combatants {
            if other_e == entity || other_species == species {
                continue;
            }
            let dist = pos.distance(*other_pos);
            if dist < combat_range {
                if nearest_enemy.is_none() || dist < nearest_enemy.unwrap().1 {
                    nearest_enemy = Some((*other_e, dist));
                }
            }
        }

        if let Some((target, _)) = nearest_enemy {
            let mut final_damage = *damage;
            if *is_berserker {
                final_damage *= 1.5;
            }
            // Random variance
            let variance = ((entity.index() as u64 ^ tick.0) % 20) as f32 / 10.0 - 1.0;
            final_damage += variance;
            final_damage = final_damage.max(0.5);

            damage_list.push((target, final_damage));
            kill_credits.push(*entity);
        }
    }

    // Apply damage
    let mut to_despawn = Vec::new();

    for (target, damage) in &damage_list {
        if let Ok((_, _, _, mut bio, _, _, traits)) = agents.get_mut(*target) {
            let effective_damage = damage / traits.defense_modifier();
            bio.health -= effective_damage;
            if bio.health <= 0.0 {
                to_despawn.push(*target);
            }
        }
    }

    // Increment kill counts
    for entity in &kill_credits {
        if let Ok((_, _, _, _, mut stats, _, _)) = agents.get_mut(*entity) {
            stats.kill_count += 1;
        }
    }

    // Despawn dead
    for entity in to_despawn {
        commands.entity(entity).despawn();
    }
}
