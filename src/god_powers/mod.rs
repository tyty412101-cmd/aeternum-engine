use bevy::prelude::*;

use crate::agents::{self, Species, AgentMarker};
use crate::world::{BiomeType, TileMarker, TileState, WorldGrid};
use crate::SimConfig;

pub struct GodPowersPlugin;

impl Plugin for GodPowersPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentGodTool(GodTool::Select))
            .add_systems(Update, god_tool_system);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GodTool {
    Select,
    RaiseLand,
    LowerLand,
    PaintBiome(BiomeType),
    SpawnEntity(Species),
    Meteor,
    Lightning,
    Fire,
    Plague,
}

#[derive(Resource)]
pub struct CurrentGodTool(pub GodTool);

fn get_world_cursor(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,
) -> Option<Vec2> {
    let window = windows.get_single().ok()?;
    let (camera, camera_transform) = cameras.get_single().ok()?;
    let cursor_pos = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}

fn god_tool_system(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,
    god_tool: Res<CurrentGodTool>,
    world_grid: Res<WorldGrid>,
    mut god_params: ParamSet<(
        Query<(&mut TileState, &mut Sprite), With<TileMarker>>,
        Query<(Entity, &Transform), With<AgentMarker>>,
    )>,
    config: Res<SimConfig>,
    mut contexts: bevy_egui::EguiContexts,
) {
    if god_tool.0 == GodTool::Select {
        return;
    }

    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    if contexts.ctx_mut().is_pointer_over_area() {
        return;
    }

    let Some(world_pos) = get_world_cursor(&windows, &cameras) else { return };
    let (gx, gy) = world_grid.world_to_grid(world_pos);

    match god_tool.0 {
        GodTool::RaiseLand | GodTool::LowerLand => {
            let radius = 3;
            let delta = if god_tool.0 == GodTool::RaiseLand { 0.05 } else { -0.05 };
            let mut tiles_q = god_params.p0();

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if let Some(entity) = world_grid.get_tile(gx + dx, gy + dy) {
                        if let Ok((mut tile, mut sprite)) = tiles_q.get_mut(entity) {
                            tile.elevation = (tile.elevation + delta).clamp(0.0, 1.0);
                            if tile.elevation < 0.3 {
                                tile.biome = BiomeType::Water;
                            } else if tile.elevation > 0.78 {
                                tile.biome = BiomeType::Mountain;
                            }
                            tile.fertility = tile.biome.fertility();
                            sprite.color = tile.biome.color();
                        }
                    }
                }
            }
        }
        GodTool::PaintBiome(biome) => {
            let radius = 2;
            let mut tiles_q = god_params.p0();
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if let Some(entity) = world_grid.get_tile(gx + dx, gy + dy) {
                        if let Ok((mut tile, mut sprite)) = tiles_q.get_mut(entity) {
                            tile.biome = biome;
                            tile.fertility = biome.fertility();
                            tile.fuel = match biome {
                                BiomeType::Forest => 0.8,
                                BiomeType::Grassland => 0.3,
                                _ => 0.0,
                            };
                            sprite.color = biome.color();
                        }
                    }
                }
            }
        }
        GodTool::SpawnEntity(species) => {
            if !mouse.just_pressed(MouseButton::Left) {
                return;
            }
            let pos = Vec2::new(world_pos.x, world_pos.y);
            let mut rng = rand::thread_rng();
            agents::spawning::spawn_agent(&mut commands, &mut rng, species, pos, &config);
        }
        GodTool::Meteor => {
            if !mouse.just_pressed(MouseButton::Left) {
                return;
            }
            let radius = 5;

            // Damage terrain
            {
                let mut tiles_q = god_params.p0();
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let dist = ((dx * dx + dy * dy) as f32).sqrt();
                        if dist > radius as f32 {
                            continue;
                        }
                        if let Some(entity) = world_grid.get_tile(gx + dx, gy + dy) {
                            if let Ok((mut tile, mut sprite)) = tiles_q.get_mut(entity) {
                                tile.biome = BiomeType::Scorched;
                                tile.fertility = 0.0;
                                tile.temperature = (tile.temperature + 0.5).min(1.0);
                                tile.on_fire = tile.fuel > 0.0;
                                if dist < radius as f32 * 0.5 {
                                    tile.elevation = (tile.elevation - 0.2).max(0.0);
                                }
                                sprite.color = BiomeType::Scorched.color();
                            }
                        }
                    }
                }
            }

            // Kill nearby agents
            {
                let agents_q = god_params.p1();
                let to_despawn: Vec<Entity> = agents_q.iter()
                    .filter(|(_, t)| world_pos.distance(t.translation.truncate()) < radius as f32 * world_grid.tile_size)
                    .map(|(e, _)| e)
                    .collect();
                for entity in to_despawn {
                    commands.entity(entity).despawn();
                }
            }
        }
        GodTool::Lightning => {
            if !mouse.just_pressed(MouseButton::Left) {
                return;
            }
            {
                let mut tiles_q = god_params.p0();
                if let Some(entity) = world_grid.get_tile(gx, gy) {
                    if let Ok((mut tile, mut sprite)) = tiles_q.get_mut(entity) {
                        tile.on_fire = tile.fuel > 0.0;
                        tile.temperature = (tile.temperature + 0.3).min(1.0);
                        sprite.color = Color::srgb(1.0, 1.0, 0.5);
                    }
                }
            }
            {
                let agents_q = god_params.p1();
                let to_despawn: Vec<Entity> = agents_q.iter()
                    .filter(|(_, t)| world_pos.distance(t.translation.truncate()) < world_grid.tile_size)
                    .map(|(e, _)| e)
                    .collect();
                for entity in to_despawn {
                    commands.entity(entity).despawn();
                }
            }
        }
        GodTool::Fire => {
            let radius = 2;
            let mut tiles_q = god_params.p0();
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if let Some(entity) = world_grid.get_tile(gx + dx, gy + dy) {
                        if let Ok((mut tile, mut sprite)) = tiles_q.get_mut(entity) {
                            if tile.fuel > 0.0 {
                                tile.on_fire = true;
                                sprite.color = Color::srgb(0.9, 0.3, 0.0);
                            }
                        }
                    }
                }
            }
        }
        GodTool::Plague => {
            if !mouse.just_pressed(MouseButton::Left) {
                return;
            }
            let plague_radius = 8.0 * world_grid.tile_size;
            let agents_q = god_params.p1();
            let to_despawn: Vec<Entity> = agents_q.iter()
                .filter(|(_, t)| {
                    world_pos.distance(t.translation.truncate()) < plague_radius
                        && rand::random::<f32>() < 0.3
                })
                .map(|(e, _)| e)
                .collect();
            for entity in to_despawn {
                commands.entity(entity).despawn();
            }
        }
        _ => {}
    }
}
