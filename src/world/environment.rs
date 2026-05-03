use bevy::prelude::*;

use super::{BiomeType, TileState, WorldGrid};

pub fn environment_update_system(
    mut tiles: Query<&mut TileState>,
    world_grid: Res<WorldGrid>,
    tick: Res<crate::SimTick>,
) {
    if tick.0 % 5 != 0 {
        return;
    }

    let width = world_grid.width;
    let height = world_grid.height;

    let tile_data: Vec<(f32, f32, f32, bool, f32, BiomeType)> = tiles
        .iter()
        .map(|t| (t.temperature, t.corruption, t.moisture, t.on_fire, t.fuel, t.biome))
        .collect();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let entity = world_grid.tiles[idx];

            if let Ok(mut tile) = tiles.get_mut(entity) {
                let mut temp_delta = 0.0f32;
                let mut corruption_delta = 0.0f32;
                let mut neighbor_count = 0;

                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx >= 0 && nx < width && ny >= 0 && ny < height {
                            let nidx = (ny * width + nx) as usize;
                            let (n_temp, n_corrupt, _, n_fire, _, _) = tile_data[nidx];
                            temp_delta += (n_temp - tile_data[idx].0) * 0.02;
                            if n_corrupt > 0.1 {
                                corruption_delta += n_corrupt * 0.005;
                            }
                            if n_fire && tile.fuel > 0.0 && !tile.on_fire {
                                if rand::random::<f32>() < 0.02 * tile.fuel {
                                    tile.on_fire = true;
                                }
                            }
                            neighbor_count += 1;
                        }
                    }
                }

                if neighbor_count > 0 {
                    tile.temperature += temp_delta / neighbor_count as f32;
                    tile.corruption += corruption_delta;
                }

                if tile.on_fire {
                    tile.fuel -= 0.01;
                    tile.temperature += 0.05;
                    tile.fertility = (tile.fertility - 0.02).max(0.0);
                    if tile.fuel <= 0.0 {
                        tile.on_fire = false;
                        tile.biome = BiomeType::Scorched;
                    }
                }

                if tile.biome == BiomeType::Scorched && !tile.on_fire {
                    tile.fertility += 0.0001;
                    if tile.fertility > 0.3 {
                        tile.biome = BiomeType::Grassland;
                        tile.fuel = 0.3;
                    }
                }

                tile.corruption = tile.corruption.clamp(0.0, 1.0);
                tile.temperature = tile.temperature.clamp(0.0, 1.0);
            }
        }
    }
}
