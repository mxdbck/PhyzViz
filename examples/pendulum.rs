use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::time::{Fixed, TimePlugin};
use bevy::color::palettes::css::*;
use bevy_vector_shapes::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use std::time::Duration;

use PhyzViz::utils::ODEs;
use PhyzViz::utils::rk4;
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::{Bloom},
};



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
    fn call(&self, t: f32, y: Vec<f32>) -> Vec<f32> {
        let theta = y[0];
        let omega = y[1];
        let dtheta_dt = omega;
        let domega_dt = -(self.gravity / self.length) * theta.sin();
        vec![dtheta_dt, domega_dt]
    }
}

/// Returns the angular displacement (radians) of the pendulum at time t (seconds).
/// Simple small-angle sinusoidal solution: theta(t) = THETA_MAX * sin(omega * t)
#[allow(dead_code)]
fn pendulum_angle(t: f32) -> f32 {
    const THETA_MAX: f32 = 0.5;      // max angle (radians) ~28.6°
    const PERIOD: f32 = 2.5;         // seconds
    let omega = std::f32::consts::TAU / PERIOD;
    THETA_MAX * (omega * t).sin()
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Tonemapping::TonyMcMapface, // 1. Using a tonemapper that desaturates to white is recommended
        Bloom::default(),           // 2. Enable bloom for the camera
        DebandDither::Enabled,      // Optional: bloom causes gradients which cause banding
    ));
    commands.insert_resource(PendulumState { theta: 3.14, omega: 0.0, params: SimplePendulum { length: 2.0, gravity: 9.81 } });
}

// Runs at your chosen Fixed timestep (e.g. 120 Hz)
fn step_pendulum(time_fixed: Res<Time<Fixed>>, mut state: ResMut<PendulumState>) {
    let dt = time_fixed.delta_secs();
    let t = time_fixed.elapsed_secs();

    let y0 = vec![state.theta, state.omega];
    let y1 = rk4::rk4(&state.params, t, y0, dt);
    state.theta = y1[0];
    state.omega = y1[1];
}

fn draw_pendulum(mut painter: ShapePainter, state: Res<PendulumState>) {
    painter.scale(Vec3::splat(80.0));

    // --- stacking (back → front) ---
    // 1) rod (z = 0.0)
    // 2) pivot circle (z = +0.001)
    // 3) bob circle (z = +0.002)

    let length: f32 = 2.0;
    let bob_radius = 0.12;

    let pivot = Vec3::ZERO;
    let theta = state.theta;
    let bob_pos = Vec3::new(length * theta.sin(), -length * theta.cos(), 0.0);

    // Save base transform
    let base = painter.transform;

    // --- rod at z = 0.0 ---
    painter.transform = base;
    painter.thickness = 0.03;
    painter.set_color(Color::WHITE);
    painter.line(pivot, bob_pos);

    // --- pivot circle at z = +0.001 ---
    let mut t = base;
    t.translation.z += 0.001;
    painter.transform = t;
    painter.thickness = 0.02;
    painter.hollow = false;
    painter.set_color(Color::srgb(0.56, 0.57, 0.64));
    painter.translate(pivot);
    painter.circle(0.07);

    // --- bob circle at z = +0.002 ---
    let mut t2 = base;
    t2.translation.z += 0.002;
    painter.transform = t2;
    painter.translate(bob_pos);
    painter.set_color(Color::linear_rgba(3.0, 0.6, 0.2, 1.0)); // bright for bloom
    painter.circle(bob_radius);

    // (optional) restore
    painter.transform = base;
}

fn main() {
    App::new()
        // Force VSync: cap to the display refresh rate
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Fifo, // classic VSync cap
                    canvas: Some("#bevy".into()),
                    // Keep the WebGL canvas exactly as big as its parent (puts the wasm module in full screen basically)
                    fit_canvas_to_parent: true,
                    resizable: true,
                    ..default()
                }),
                ..default()
            })
            // Make sure FixedUpdate is enabled (it is by default, but showing explicitly)
            .set(TimePlugin::default()),
        )
        // Set physics tick rate (e.g., 120 Hz)
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(1.0 / 120.0)))
        .add_plugins(Shape2dPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ClearColor(bevy::prelude::Color::Srgba(BLACK)))
        .add_systems(Startup, setup)
        // Physics on a fixed timestep
        .add_systems(FixedUpdate, step_pendulum)
        // Rendering on the variable-rate Update schedule (interpolation optional)
        .add_systems(Update, draw_pendulum)
        .run();
}
