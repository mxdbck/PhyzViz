use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::time::{Fixed, TimePlugin};
use bevy_vector_shapes::prelude::*;
use std::time::Duration;

use PhyzViz::utils::ODEs;
use PhyzViz::utils::rk4;
use PhyzViz::utils::mesh_ribbon::{spawn_mesh_ribbon, MeshRibbonParams, add_ribbon_position};
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::{Bloom},
};

#[cfg(feature = "fps_overlay")]
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;

const RENDER_SCALE: f32 = 60.0;

struct SimplePendulum {
    length: f32,
    gravity: f32,
}

#[derive(Resource)]
struct PendulumState {
    theta: f32,
    omega: f32,
    params: SimplePendulum
}

impl ODEs::ODEFunc for SimplePendulum {
    fn call(&self, _t: f32, y: Vec<f32>) -> Vec<f32> {
        let theta = y[0];
        let omega = y[1];
        let dtheta_dt = omega;
        let domega_dt = -(self.gravity / self.length) * theta.sin();
        vec![dtheta_dt, domega_dt]
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Tonemapping::TonyMcMapface,
        Bloom::default(),
        DebandDither::Enabled,
    ));
    commands.insert_resource(PendulumState { theta: 2.5, omega: 0.0, params: SimplePendulum { length: 2.0, gravity: 9.81 } });

    // Spawn mesh ribbon
    spawn_mesh_ribbon(&mut commands, &mut meshes, &mut materials, "bob_mesh_ribbon".to_string(), MeshRibbonParams {
        width: 3.0,
        max_points: 1000,
        color: Color::linear_rgba(10.0, 8.7, 10.0, 1.0),
        fade_to_transparent: true,
    });

    // TODO: Create a simple-pendulum.png with the equation and add it to assets/
    // For now, this will fail gracefully if the file doesn't exist
    let mut sprite = Sprite::from_image(asset_server.load("simple-pendulum.png"));
    sprite.color = Color::Srgba(Srgba { red: 1.5, green: 1.5, blue: 1.5, alpha: 1.0 });
    commands.spawn((
        sprite,
        Transform::from_xyz(0.0, 250.0, -1.0).with_scale(Vec3::splat(0.15)),
    ));
}

fn step_pendulum(time_fixed: Res<Time<Fixed>>, mut state: ResMut<PendulumState>) {
    let dt = time_fixed.delta_secs() / 2.0;
    let t = time_fixed.elapsed_secs() / 2.0;

    let y0 = vec![state.theta, state.omega];
    let y1 = rk4::rk4(&state.params, t, y0, dt);
    state.theta = y1[0];
    state.omega = y1[1];
}

fn draw_pendulum(
    mut painter: ShapePainter,
    state: Res<PendulumState>,
    mut q_mesh: Query<(&mut PhyzViz::utils::mesh_ribbon::MeshRibbon, &Name)>,
) {
    painter.scale(Vec3::splat(RENDER_SCALE));

    let length: f32 = 2.0;
    let bob_radius = 0.12;

    let pivot = Vec3::ZERO;
    let theta = state.theta;
    let bob_pos = Vec3::new(length * theta.sin(), -length * theta.cos(), 0.0);

    let base = painter.transform;

    // --- rod at z = 0.0 ---
    painter.transform = base;
    painter.thickness = 0.05;
    painter.set_color(Srgba { red: 4.0 * 165.0 / 255.0, green: 4.0 * 136.0 / 255.0, blue: 4.0 * 94.0 / 255.0, alpha: 1.0 });
    painter.line(pivot, bob_pos);

    // --- pivot circle at z = +0.001 ---
    let mut t = base;
    t.translation.z += 0.001;
    painter.transform = t;
    painter.thickness = 0.02;
    painter.hollow = false;
    painter.set_color(Srgba { red: 4.0 * 165.0 / 255.0, green: 4.0 * 136.0 / 255.0, blue: 4.0 * 94.0 / 255.0, alpha: 1.0 });
    painter.translate(pivot);
    painter.circle(0.07);

    // --- bob circle at z = +0.002 ---
    let mut t2 = base;
    t2.translation.z += 0.002;
    painter.transform = t2;
    painter.translate(bob_pos);
    painter.set_color(Color::linear_rgba(3.0, 0.6, 0.2, 1.0));
    painter.circle(bob_radius);

    painter.transform = base;

    // Move mesh ribbon position
    for (mut ribbon, name) in q_mesh.iter_mut() {
        if name.as_str() == "bob_mesh_ribbon" {
            ribbon.current_position = bob_pos * RENDER_SCALE;
        }
    }
}

fn main() {
    let mut app = App::new();
    
    app
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoVsync,

                    #[cfg(target_arch = "wasm32")]
                    canvas: Some("#bevy".into()),
                    #[cfg(target_arch = "wasm32")]
                    fit_canvas_to_parent: true,

                    resizable: true,
                    ..default()
                }),
                ..default()
            })
            .set(TimePlugin::default()),
        )
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(1.0 / 120.0)))
        .add_plugins(Shape2dPlugin::default())
        .insert_resource(ClearColor(bevy::prelude::Color::Srgba(Srgba { red: 84.0 / 255.0, green: 18.0 / 255.0, blue: 18.0 / 255.0, alpha: 1.0 })))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, step_pendulum)
        .add_systems(Update, draw_pendulum)
        .add_systems(Update, add_ribbon_position);

    #[cfg(feature = "fps_overlay")]
    app.add_plugins(FpsOverlayPlugin::default());

    app.run();
}
