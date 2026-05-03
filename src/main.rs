mod world;
mod agents;
mod traits;
mod civilization;
mod combat;
mod ui;
mod god_powers;

use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::window::PresentMode;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Aeternum Engine — Simulated Chaos, Bound by Rules".to_string(),
                resolution: (1280.0, 720.0).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(SimConfig::default())
        .insert_resource(SimTick(0))
        .insert_resource(TimeScale(1.0))
        .insert_resource(SelectedEntity(None))
        .insert_resource(GameCamera {
            zoom: 1.0,
            target_zoom: 1.0,
        })
        .insert_resource(Time::<Fixed>::from_hz(30.0))
        .add_plugins((
            world::WorldPlugin,
            agents::AgentPlugin,
            civilization::CivilizationPlugin,
            combat::CombatPlugin,
            ui::UiPlugin,
            god_powers::GodPowersPlugin,
        ))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (
            camera_movement_system,
            camera_zoom_system,
            tick_system,
            tile_visual_update_system,
            keyboard_shortcuts,
        ))
        .run();
}

#[derive(Resource)]
pub struct SimConfig {
    pub world_width: i32,
    pub world_height: i32,
    pub tile_size: f32,
    pub seed: u64,
    pub initial_population_per_species: u32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            world_width: 128,
            world_height: 128,
            tile_size: 8.0,
            seed: 42,
            initial_population_per_species: 30,
        }
    }
}

#[derive(Resource)]
pub struct SimTick(pub u64);

#[derive(Resource)]
pub struct TimeScale(pub f32);

#[derive(Resource)]
pub struct SelectedEntity(pub Option<Entity>);

#[derive(Resource)]
pub struct GameCamera {
    pub zoom: f32,
    pub target_zoom: f32,
}

#[derive(Component)]
pub struct MainCamera;

fn setup_camera(mut commands: Commands, config: Res<SimConfig>) {
    let center_x = config.world_width as f32 * config.tile_size * 0.5;
    let center_y = config.world_height as f32 * config.tile_size * 0.5;

    commands.spawn((
        Camera2d,
        Transform::from_xyz(center_x, center_y, 999.0),
        MainCamera,
    ));
}

fn camera_movement_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cameras: Query<&mut Transform, With<MainCamera>>,
    game_camera: Res<GameCamera>,
    time: Res<Time>,
) {
    let Ok(mut transform) = cameras.get_single_mut() else { return };

    let speed = 300.0 * game_camera.zoom * time.delta_secs();

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
        transform.translation.x += direction.x * speed;
        transform.translation.y += direction.y * speed;
    }
}

fn camera_zoom_system(
    mut scroll_events: EventReader<MouseWheel>,
    mut cameras: Query<&mut OrthographicProjection, With<MainCamera>>,
    mut game_camera: ResMut<GameCamera>,
) {
    let Ok(mut projection) = cameras.get_single_mut() else { return };

    for event in scroll_events.read() {
        let zoom_delta = -event.y * 0.1;
        game_camera.target_zoom = (game_camera.target_zoom + zoom_delta).clamp(0.2, 15.0);
    }

    // Smooth zoom
    let diff = game_camera.target_zoom - game_camera.zoom;
    game_camera.zoom += diff * 0.1;
    projection.scale = game_camera.zoom;
}

fn tick_system(
    mut tick: ResMut<SimTick>,
    time_scale: Res<TimeScale>,
) {
    if time_scale.0 > 0.0 {
        tick.0 += 1;
    }
}

fn tile_visual_update_system(
    mut tiles: Query<(&world::TileState, &mut Sprite), With<world::TileMarker>>,
    tick: Res<SimTick>,
) {
    if tick.0 % 30 != 0 {
        return;
    }

    for (tile, mut sprite) in tiles.iter_mut() {
        let base_color = tile.biome.color();

        if tile.on_fire {
            let flicker = ((tick.0 as f32 * 0.3).sin() * 0.5 + 0.5) * 0.3;
            sprite.color = Color::srgb(
                0.9 + flicker,
                0.2 + flicker * 0.5,
                0.0,
            );
        } else if tile.corruption > 0.1 {
            let srgba = base_color.to_srgba();
            sprite.color = Color::srgb(
                srgba.red * (1.0 - tile.corruption * 0.5),
                srgba.green * (1.0 - tile.corruption * 0.7),
                srgba.blue * (1.0 - tile.corruption * 0.3) + tile.corruption * 0.2,
            );
        } else {
            sprite.color = base_color;
        }
    }
}

fn keyboard_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut time_scale: ResMut<TimeScale>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        if time_scale.0 > 0.0 {
            time_scale.0 = 0.0;
        } else {
            time_scale.0 = 1.0;
        }
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        time_scale.0 = 0.5;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        time_scale.0 = 1.0;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        time_scale.0 = 3.0;
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        time_scale.0 = 10.0;
    }
}
