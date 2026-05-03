pub mod ai;
pub mod spawning;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::traits::TraitSet;

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawning::spawn_initial_populations)
            .add_systems(FixedUpdate, (
                ai::agent_perception_system,
                ai::agent_decision_system,
                ai::agent_action_system,
                ai::agent_aging_system,
                ai::agent_hunger_system,
            ).chain());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Species {
    Human,
    Elf,
    Orc,
    Dwarf,
}

impl Species {
    pub fn base_stats(&self) -> BaseStats {
        match self {
            Species::Human => BaseStats {
                max_health: 100.0,
                strength: 10.0,
                speed: 10.0,
                intelligence: 10.0,
                aggression: 0.5,
                max_age: 80_000,
                fertility_rate: 0.5,
                perception_radius: 8.0,
            },
            Species::Elf => BaseStats {
                max_health: 80.0,
                strength: 7.0,
                speed: 12.0,
                intelligence: 14.0,
                aggression: 0.3,
                max_age: 500_000,
                fertility_rate: 0.2,
                perception_radius: 12.0,
            },
            Species::Orc => BaseStats {
                max_health: 140.0,
                strength: 15.0,
                speed: 8.0,
                intelligence: 6.0,
                aggression: 0.9,
                max_age: 60_000,
                fertility_rate: 0.7,
                perception_radius: 6.0,
            },
            Species::Dwarf => BaseStats {
                max_health: 120.0,
                strength: 13.0,
                speed: 7.0,
                intelligence: 9.0,
                aggression: 0.5,
                max_age: 150_000,
                fertility_rate: 0.3,
                perception_radius: 7.0,
            },
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Species::Human => Color::srgb(0.9, 0.75, 0.55),
            Species::Elf => Color::srgb(0.6, 0.9, 0.65),
            Species::Orc => Color::srgb(0.4, 0.7, 0.35),
            Species::Dwarf => Color::srgb(0.8, 0.55, 0.35),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Species::Human => "Human",
            Species::Elf => "Elf",
            Species::Orc => "Orc",
            Species::Dwarf => "Dwarf",
        }
    }
}

pub struct BaseStats {
    pub max_health: f32,
    pub strength: f32,
    pub speed: f32,
    pub intelligence: f32,
    pub aggression: f32,
    pub max_age: u32,
    pub fertility_rate: f32,
    pub perception_radius: f32,
}

#[derive(Component, Clone, Debug)]
pub struct Agent {
    pub species: Species,
    pub name: String,
}

#[derive(Component, Clone, Debug)]
pub struct BiologicalState {
    pub health: f32,
    pub max_health: f32,
    pub health_regen: f32,
    pub stamina: f32,
    pub hunger: f32,
    pub age: u32,
    pub max_age: u32,
    pub reproduction_cooldown: u32,
}

#[derive(Component, Clone, Debug)]
pub struct CognitiveState {
    pub threat_level: f32,
    pub opportunity_score: f32,
    pub risk_tolerance: f32,
    pub decision_latency: u8,
    pub focus_target: Option<Entity>,
    pub current_action: AgentAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AgentAction {
    Idle,
    Wander,
    SeekFood,
    Eat,
    Rest,
    Work,
    Build,
    Fight,
    Flee,
    Explore,
    Socialize,
    MoveToTarget,
}

impl Default for AgentAction {
    fn default() -> Self {
        AgentAction::Idle
    }
}

#[derive(Component, Clone, Debug)]
pub struct AgentStats {
    pub strength: f32,
    pub speed: f32,
    pub intelligence: f32,
    pub aggression: f32,
    pub perception_radius: f32,
    pub kill_count: u32,
    pub children_count: u32,
}

#[derive(Component, Clone, Debug)]
pub struct Allegiance {
    pub kingdom: Option<Entity>,
    pub settlement: Option<Entity>,
    pub loyalty: f32,
    pub role: AgentRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentRole {
    Villager,
    Worker,
    Soldier,
    Scout,
    Leader,
}

#[derive(Component)]
pub struct MoveTarget {
    pub target: Vec2,
    pub arrived: bool,
}

#[derive(Component)]
pub struct AgentMarker;
