use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

use super::{BiomeType, TileMarker, TileState, WorldGrid};
use crate::SimConfig;

pub fn generate_world(
    mut commands: Commands,
    config: Res<SimConfig>,
) {
    let width = config.world_width;
    let height = config.world_height;
    let tile_size = config.tile_size;

    let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
    let elevation_noise = Perlin::new(rng.gen());
    let moisture_noise = Perlin::new(rng.gen());
    let temperature_noise = Perlin::new(rng.gen());

    let mut tiles = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 / width as f64;
            let ny = y as f64 / height as f64;

            let elevation = sample_octaves(&elevation_noise, nx, ny, 6, 2.0, 0.5);
            let moisture = sample_octaves(&moisture_noise, nx + 100.0, ny + 100.0, 4, 2.0, 0.5);
            let base_temp = 1.0 - (ny - 0.5).abs() * 2.0;
            let temp_noise = sample_octaves(&temperature_noise, nx + 200.0, ny + 200.0, 3, 2.0, 0.5) * 0.3;
            let temperature = (base_temp as f32 + temp_noise as f32 - elevation as f32 * 0.5).clamp(0.0, 1.0);

            let biome = classify_biome(elevation as f32, moisture as f32, temperature);

            let fuel = match biome {
                BiomeType::Forest => 0.8,
                BiomeType::Grassland => 0.3,
                BiomeType::Swamp => 0.1,
                _ => 0.0,
            };

            let tile_state = TileState {
                biome,
                elevation: elevation as f32,
                temperature,
                moisture: moisture as f32,
                fertility: biome.fertility(),
                corruption: 0.0,
                fuel,
                on_fire: false,
                has_lava: false,
                grid_x: x,
                grid_y: y,
            };

            let world_x = x as f32 * tile_size + tile_size * 0.5;
            let world_y = y as f32 * tile_size + tile_size * 0.5;

            let entity = commands
                .spawn((
                    Sprite {
                        color: biome.color(),
                        custom_size: Some(Vec2::splat(tile_size)),
                        ..default()
                    },
                    Transform::from_xyz(world_x, world_y, 0.0),
                    tile_state,
                    TileMarker,
                ))
                .id();

            tiles.push(entity);
        }
    }

    commands.insert_resource(WorldGrid {
        width,
        height,
        tiles,
        tile_size,
    });
}

fn sample_octaves(noise: &Perlin, x: f64, y: f64, octaves: u32, lacunarity: f64, persistence: f64) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 3.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += noise.get([x * frequency, y * frequency]) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    (value / max_value + 1.0) * 0.5
}

fn classify_biome(elevation: f32, moisture: f32, temperature: f32) -> BiomeType {
    if elevation < 0.3 {
        return BiomeType::Water;
    }

    if elevation > 0.78 {
        if temperature < 0.2 {
            return BiomeType::Tundra;
        }
        return BiomeType::Mountain;
    }

    if temperature < 0.2 {
        return BiomeType::Tundra;
    }

    if moisture > 0.65 && temperature > 0.4 {
        if elevation < 0.4 {
            return BiomeType::Swamp;
        }
        return BiomeType::Forest;
    }

    if moisture > 0.45 {
        return BiomeType::Forest;
    }

    if temperature > 0.65 && moisture < 0.35 {
        return BiomeType::Desert;
    }

    BiomeType::Grassland
}
