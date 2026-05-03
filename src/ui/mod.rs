use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crate::agents::*;
use crate::civilization::*;
use crate::traits::TraitSet;
use crate::world::{WorldGrid, TileState};
use crate::{SimConfig, SimTick, TimeScale, SelectedEntity, GameCamera};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Update, top_bar_ui)
            .add_systems(Update, left_panel_ui)
            .add_systems(Update, right_panel_ui)
            .add_systems(Update, selection_system);
    }
}

fn top_bar_ui(
    mut contexts: EguiContexts,
    agents: Query<&Agent, With<AgentMarker>>,
    settlements: Query<&Settlement, With<SettlementMarker>>,
    kingdoms: Query<&Kingdom, With<KingdomMarker>>,
    tick: Res<SimTick>,
    mut time_scale: ResMut<TimeScale>,
) {
    let agent_count = agents.iter().count();
    let settlement_count = settlements.iter().count();
    let kingdom_count = kingdoms.iter().count();

    let mut humans = 0u32;
    let mut elves = 0u32;
    let mut orcs = 0u32;
    let mut dwarves = 0u32;

    for agent in agents.iter() {
        match agent.species {
            Species::Human => humans += 1,
            Species::Elf => elves += 1,
            Species::Orc => orcs += 1,
            Species::Dwarf => dwarves += 1,
        }
    }

    egui::TopBottomPanel::top("top_bar").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.heading("Aeternum Engine");
            ui.separator();

            if ui.button("⏸").clicked() {
                time_scale.0 = 0.0;
            }
            if ui.button("▶").clicked() {
                time_scale.0 = 1.0;
            }
            if ui.button("▶▶").clicked() {
                time_scale.0 = 3.0;
            }
            if ui.button("▶▶▶").clicked() {
                time_scale.0 = 10.0;
            }

            ui.label(format!("Speed: {:.0}x", time_scale.0));
            ui.separator();

            ui.label(format!("Tick: {}", tick.0));
            ui.separator();
            ui.label(format!("Pop: {}", agent_count));
            ui.label(format!("(H:{} E:{} O:{} D:{})", humans, elves, orcs, dwarves));
            ui.separator();
            ui.label(format!("Settlements: {}", settlement_count));
            ui.label(format!("Kingdoms: {}", kingdom_count));
        });
    });
}

fn left_panel_ui(
    mut contexts: EguiContexts,
    mut god_tool: ResMut<crate::god_powers::CurrentGodTool>,
) {
    egui::SidePanel::left("tools_panel").min_width(140.0).show(contexts.ctx_mut(), |ui| {
        ui.heading("God Powers");
        ui.separator();

        ui.label("Terrain");
        if ui.button("Raise Land").clicked() {
            god_tool.0 = crate::god_powers::GodTool::RaiseLand;
        }
        if ui.button("Lower Land").clicked() {
            god_tool.0 = crate::god_powers::GodTool::LowerLand;
        }

        ui.separator();
        ui.label("Biomes");
        if ui.button("Paint Grass").clicked() {
            god_tool.0 = crate::god_powers::GodTool::PaintBiome(crate::world::BiomeType::Grassland);
        }
        if ui.button("Paint Forest").clicked() {
            god_tool.0 = crate::god_powers::GodTool::PaintBiome(crate::world::BiomeType::Forest);
        }
        if ui.button("Paint Desert").clicked() {
            god_tool.0 = crate::god_powers::GodTool::PaintBiome(crate::world::BiomeType::Desert);
        }
        if ui.button("Paint Water").clicked() {
            god_tool.0 = crate::god_powers::GodTool::PaintBiome(crate::world::BiomeType::Water);
        }

        ui.separator();
        ui.label("Life");
        if ui.button("Spawn Human").clicked() {
            god_tool.0 = crate::god_powers::GodTool::SpawnEntity(Species::Human);
        }
        if ui.button("Spawn Elf").clicked() {
            god_tool.0 = crate::god_powers::GodTool::SpawnEntity(Species::Elf);
        }
        if ui.button("Spawn Orc").clicked() {
            god_tool.0 = crate::god_powers::GodTool::SpawnEntity(Species::Orc);
        }
        if ui.button("Spawn Dwarf").clicked() {
            god_tool.0 = crate::god_powers::GodTool::SpawnEntity(Species::Dwarf);
        }

        ui.separator();
        ui.label("Disasters");
        if ui.button("Meteor ☄️").clicked() {
            god_tool.0 = crate::god_powers::GodTool::Meteor;
        }
        if ui.button("Lightning ⚡").clicked() {
            god_tool.0 = crate::god_powers::GodTool::Lightning;
        }
        if ui.button("Fire 🔥").clicked() {
            god_tool.0 = crate::god_powers::GodTool::Fire;
        }
        if ui.button("Plague 🦠").clicked() {
            god_tool.0 = crate::god_powers::GodTool::Plague;
        }

        ui.separator();
        if ui.button("Select (default)").clicked() {
            god_tool.0 = crate::god_powers::GodTool::Select;
        }
    });
}

fn right_panel_ui(
    mut contexts: EguiContexts,
    selected: Res<SelectedEntity>,
    agents: Query<(&Agent, &BiologicalState, &CognitiveState, &AgentStats, &TraitSet, &Allegiance), With<AgentMarker>>,
    settlements: Query<(&Settlement, &Transform), With<SettlementMarker>>,
    kingdoms: Query<(&Kingdom, &DiplomacyState), With<KingdomMarker>>,
    tiles: Query<&TileState>,
) {
    egui::SidePanel::right("info_panel").min_width(200.0).show(contexts.ctx_mut(), |ui| {
        if let Some(entity) = selected.0 {
            // Try agent
            if let Ok((agent, bio, cognitive, stats, traits, allegiance)) = agents.get(entity) {
                ui.heading(format!("{}", agent.name));
                ui.label(format!("Species: {}", agent.species.name()));
                ui.separator();

                ui.label(format!("Health: {:.0}/{:.0}", bio.health, bio.max_health));
                ui.label(format!("Hunger: {:.0}%", bio.hunger * 100.0));
                ui.label(format!("Stamina: {:.0}%", bio.stamina * 100.0));
                ui.label(format!("Age: {}", bio.age));
                ui.separator();

                ui.label(format!("Action: {:?}", cognitive.current_action));
                ui.label(format!("Threat: {:.2}", cognitive.threat_level));
                ui.separator();

                ui.label(format!("Strength: {:.1}", stats.strength));
                ui.label(format!("Speed: {:.1}", stats.speed));
                ui.label(format!("Intelligence: {:.1}", stats.intelligence));
                ui.label(format!("Kills: {}", stats.kill_count));
                ui.separator();

                ui.label("Traits:");
                for t in &traits.traits {
                    ui.colored_label(
                        egui::Color32::from_rgb(
                            (t.color().to_srgba().red * 255.0) as u8,
                            (t.color().to_srgba().green * 255.0) as u8,
                            (t.color().to_srgba().blue * 255.0) as u8,
                        ),
                        format!("  [{}]", t.display_name()),
                    );
                }

                ui.separator();
                ui.label(format!("Role: {:?}", allegiance.role));
                ui.label(format!("Loyalty: {:.0}%", allegiance.loyalty * 100.0));
            }
            // Try settlement
            else if let Ok((settlement, _)) = settlements.get(entity) {
                ui.heading(&settlement.name);
                ui.label(format!("Species: {}", settlement.species.name()));
                ui.separator();
                ui.label(format!("Population: {}", settlement.population));
                ui.label(format!("Food: {:.0}", settlement.food_stockpile));
                ui.label(format!("Materials: {:.0}", settlement.material_stockpile));
                ui.label(format!("Territory: {:.0}", settlement.territory_radius));
            }
            // Try tile
            else if let Ok(tile) = tiles.get(entity) {
                ui.heading("Tile Info");
                ui.label(format!("Biome: {:?}", tile.biome));
                ui.label(format!("Position: ({}, {})", tile.grid_x, tile.grid_y));
                ui.separator();
                ui.label(format!("Elevation: {:.2}", tile.elevation));
                ui.label(format!("Temperature: {:.2}", tile.temperature));
                ui.label(format!("Moisture: {:.2}", tile.moisture));
                ui.label(format!("Fertility: {:.2}", tile.fertility));
                ui.label(format!("Corruption: {:.2}", tile.corruption));
                if tile.on_fire {
                    ui.colored_label(egui::Color32::RED, "ON FIRE!");
                }
            }
        } else {
            ui.heading("No Selection");
            ui.label("Click on a unit, settlement,");
            ui.label("or tile to inspect it.");
            ui.separator();
            ui.label("Controls:");
            ui.label("  WASD/Arrows: Pan camera");
            ui.label("  Scroll: Zoom in/out");
            ui.label("  Click: Select/Use tool");
            ui.label("  1-4: Speed controls");
        }
    });
}

fn selection_system(
    mut selected: ResMut<SelectedEntity>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,
    agents: Query<(Entity, &Transform), With<AgentMarker>>,
    settlements: Query<(Entity, &Transform), With<SettlementMarker>>,
    tiles: Query<(Entity, &Transform), With<crate::world::TileMarker>>,
    god_tool: Res<crate::god_powers::CurrentGodTool>,
    config: Res<SimConfig>,
    mut contexts: EguiContexts,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // Don't select if clicking on UI
    if contexts.ctx_mut().is_pointer_over_area() {
        return;
    }

    if god_tool.0 != crate::god_powers::GodTool::Select {
        return;
    }

    let Ok(window) = windows.get_single() else { return };
    let Ok((camera, camera_transform)) = cameras.get_single() else { return };

    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    let click_radius = config.tile_size * 0.5;

    // Check agents first (higher priority)
    let mut closest: Option<(Entity, f32)> = None;
    for (entity, transform) in agents.iter() {
        let dist = world_pos.distance(transform.translation.truncate());
        if dist < click_radius {
            if closest.is_none() || dist < closest.unwrap().1 {
                closest = Some((entity, dist));
            }
        }
    }

    if let Some((entity, _)) = closest {
        selected.0 = Some(entity);
        return;
    }

    // Check settlements
    for (entity, transform) in settlements.iter() {
        let dist = world_pos.distance(transform.translation.truncate());
        if dist < config.tile_size * 2.0 {
            selected.0 = Some(entity);
            return;
        }
    }

    // Check tiles
    for (entity, transform) in tiles.iter() {
        let dist = world_pos.distance(transform.translation.truncate());
        if dist < config.tile_size * 0.5 {
            selected.0 = Some(entity);
            return;
        }
    }

    selected.0 = None;
}
