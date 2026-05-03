pub mod generation;
pub mod environment;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, generation::generate_world)
            .add_systems(FixedUpdate, environment::environment_update_system);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    Grassland,
    Forest,
    Desert,
    Mountain,
    Water,
    Swamp,
    Tundra,
    Volcanic,
    Scorched,
}

impl BiomeType {
    pub fn color(&self) -> Color {
        match self {
            BiomeType::Grassland => Color::srgb(0.35, 0.65, 0.25),
            BiomeType::Forest => Color::srgb(0.15, 0.45, 0.12),
            BiomeType::Desert => Color::srgb(0.85, 0.75, 0.45),
            BiomeType::Mountain => Color::srgb(0.55, 0.52, 0.50),
            BiomeType::Water => Color::srgb(0.15, 0.35, 0.70),
            BiomeType::Swamp => Color::srgb(0.30, 0.40, 0.25),
            BiomeType::Tundra => Color::srgb(0.80, 0.85, 0.90),
            BiomeType::Volcanic => Color::srgb(0.35, 0.15, 0.10),
            BiomeType::Scorched => Color::srgb(0.25, 0.20, 0.15),
        }
    }

    pub fn fertility(&self) -> f32 {
        match self {
            BiomeType::Grassland => 0.8,
            BiomeType::Forest => 0.5,
            BiomeType::Desert => 0.1,
            BiomeType::Mountain => 0.0,
            BiomeType::Water => 0.0,
            BiomeType::Swamp => 0.4,
            BiomeType::Tundra => 0.1,
            BiomeType::Volcanic => 0.0,
            BiomeType::Scorched => 0.0,
        }
    }

    pub fn movement_cost(&self) -> f32 {
        match self {
            BiomeType::Grassland => 1.0,
            BiomeType::Forest => 1.3,
            BiomeType::Desert => 1.5,
            BiomeType::Mountain => 2.5,
            BiomeType::Water => 99.0,
            BiomeType::Swamp => 2.0,
            BiomeType::Tundra => 1.8,
            BiomeType::Volcanic => 3.0,
            BiomeType::Scorched => 1.2,
        }
    }

    pub fn is_passable(&self) -> bool {
        !matches!(self, BiomeType::Water | BiomeType::Mountain)
    }
}

#[derive(Component, Clone, Debug)]
pub struct TileState {
    pub biome: BiomeType,
    pub elevation: f32,
    pub temperature: f32,
    pub moisture: f32,
    pub fertility: f32,
    pub corruption: f32,
    pub fuel: f32,
    pub on_fire: bool,
    pub has_lava: bool,
    pub grid_x: i32,
    pub grid_y: i32,
}

#[derive(Component)]
pub struct TileMarker;

#[derive(Resource)]
pub struct WorldGrid {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Entity>,
    pub tile_size: f32,
}

impl WorldGrid {
    pub fn get_tile(&self, x: i32, y: i32) -> Option<Entity> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            Some(self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    pub fn get_neighbors(&self, x: i32, y: i32) -> Vec<Entity> {
        let mut neighbors = Vec::with_capacity(8);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if let Some(e) = self.get_tile(x + dx, y + dy) {
                    neighbors.push(e);
                }
            }
        }
        neighbors
    }

    pub fn world_to_grid(&self, world_pos: Vec2) -> (i32, i32) {
        let x = (world_pos.x / self.tile_size).floor() as i32;
        let y = (world_pos.y / self.tile_size).floor() as i32;
        (x, y)
    }

    pub fn grid_to_world(&self, gx: i32, gy: i32) -> Vec2 {
        Vec2::new(
            gx as f32 * self.tile_size + self.tile_size * 0.5,
            gy as f32 * self.tile_size + self.tile_size * 0.5,
        )
    }
}
