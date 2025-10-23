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

const RENDER_SCALE: f32 = 80.0;

pub struct DoublePendulum {
    pub m1: f32,
    pub m2: f32,
    pub l1: f32,
    pub l2: f32,
    pub g: f32,
}

#[derive(Resource)]
struct PendulumState {
    theta1: f32,       // Angular displacement of the first pendulum (radians)
    omega1: f32,       // Angular velocity of the first pendulum (radians/s)
    theta2: f32,       // Angular displacement of the second pendulum (radians)
    omega2: f32,       // Angular velocity of the second pendulum (radians/s)
    params: DoublePendulum
}

// Source : https://web.mit.edu/jorloff/www/chaosTalk/double-pendulum/double-pendulum-en.html
impl ODEs::ODEFunc for DoublePendulum {
    fn call(&self, _t: f32, y: Vec<f32>) -> Vec<f32> {
        // State variables
        let theta1 = y[0];
        let omega1 = y[1];
        let theta2 = y[2];
        let omega2 = y[3];

        let m1 = self.m1;
        let m2 = self.m2;
        let l1 = self.l1;
        let l2 = self.l2;
        let g = self.g;

        // Common terms
        let delta = theta1 - theta2;
        let denom = 2.0 * m1 + m2 - m2 * (2.0 * theta1 - 2.0 * theta2).cos();

        // Equations of motion
        let dtheta1_dt = omega1;
        let dtheta2_dt = omega2;

        let domega1_dt = (
            -g * (2.0 * m1 + m2) * theta1.sin()
            - m2 * g * (theta1 - 2.0 * theta2).sin()
            - 2.0 * m2 * delta.sin()
                * (omega2.powi(2) * l2 + omega1.powi(2) * l1 * delta.cos())
        ) / (l1 * denom);

        let domega2_dt = (
            2.0 * delta.sin()
                * (omega1.powi(2) * l1 * (m1 + m2)
                + g * (m1 + m2) * theta1.cos()
                + omega2.powi(2) * l2 * m2 * delta.cos())
        ) / (l2 * denom);

        vec![dtheta1_dt, domega1_dt, dtheta2_dt, domega2_dt]
    }
}


fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn((
        Camera2d,
        Tonemapping::TonyMcMapface, // 1. Using a tonemapper that desaturates to white is recommended
        Bloom::default(),           // 2. Enable bloom for the camera
        DebandDither::Enabled,      // Optional: bloom causes gradients which cause banding
    ));
    commands.insert_resource(PendulumState { theta1: 2.899002795870406, omega1: 0.0, theta2: 1.913720799888307, omega2: 0.0, params: DoublePendulum { m1: 1.0, m2: 1.0, l1: 1.0, l2: 1.0, g: 9.81 } });

    // Spawn mesh ribbons (comment out particle ribbons to compare)
    spawn_mesh_ribbon(&mut commands, &mut meshes, &mut materials, "bob1_mesh_ribbon".to_string(), MeshRibbonParams {
        width: 3.0,
        max_points: 1000,
        color: Color::linear_rgba(10.0, 8.7, 10.0, 1.0),
        fade_to_transparent: true,
    });
    spawn_mesh_ribbon(&mut commands, &mut meshes, &mut materials, "bob2_mesh_ribbon".to_string(), MeshRibbonParams {
        width: 3.0,
        max_points: 1000,
        color: Color::linear_rgba(10.0, 8.7, 10.0, 1.0),
        fade_to_transparent: true,
    });
}


fn step_pendulum(time_fixed: Res<Time<Fixed>>, mut state: ResMut<PendulumState>) {
    let dt = time_fixed.delta_secs() / 2.0;
    let t = time_fixed.elapsed_secs() / 2.0;

    let y0 = vec![state.theta1, state.omega1, state.theta2, state.omega2];

    let y1 = rk4::rk4(&state.params, t, y0, dt);
    state.theta1 = y1[0];
    state.omega1 = y1[1];
    state.theta2 = y1[2];
    state.omega2 = y1[3];
}


fn draw_pendulum(
    mut painter: ShapePainter,
    state: Res<PendulumState>,
    mut q_mesh: Query<(&mut PhyzViz::utils::mesh_ribbon::MeshRibbon, &Name)>,
) {
    painter.scale(Vec3::splat(RENDER_SCALE));

    // --- desired stacking (back → front) ---
    // 1) rod (z = 0.0)
    // 2) pivot circle (z = +0.001)
    // 3) bob circle (z = +0.002)

    let length1: f32 = 2.0;
    let length2: f32 = 2.0;
    let bob_radius = 0.12;
    
    let pivot = Vec3::ZERO;
    let theta1 = state.theta1;
    let bob1_pos = Vec3::new(length1 * theta1.sin(), -length1 * theta1.cos(), 0.0);

    let theta2 = state.theta2;
    let bob2_pos = Vec3::new(length2 * theta2.sin(), -length2 * theta2.cos(), 0.0);

    // Save base transform
    let base = painter.transform;

    // --- rod at z = 0.0 ---
    painter.transform = base;
    painter.thickness = 0.05;
    painter.set_color(Srgba { red: 4.0 * 165.0 / 255.0, green: 4.0 * 136.0 / 255.0, blue: 4.0 * 94.0 / 255.0, alpha: 1.0 });
    painter.line(pivot, bob1_pos);

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
    painter.translate(bob1_pos);
    painter.set_color(Color::linear_rgba(3.0, 0.6, 0.2, 1.0)); // bright for bloom
    painter.circle(bob_radius);

    // --- rod 2 at z = 0.0 ---
    painter.transform = base;
    painter.thickness = 0.03;
    painter.set_color(Srgba { red: 4.0 * 165.0 / 255.0, green: 4.0 * 136.0 / 255.0, blue: 4.0 * 94.0 / 255.0, alpha: 1.0 });
    painter.line(bob1_pos, bob1_pos + bob2_pos);

    // --- bob circle 2 at z = +0.002 ---
    let mut t3 = base;
    t3.translation.z += 0.002;
    painter.transform = t3;
    painter.translate(bob1_pos + bob2_pos);
    painter.set_color(Color::linear_rgba(3.0, 0.6, 0.2, 1.0)); // bright for bloom
    painter.circle(bob_radius);

    // (optional) restore
    painter.transform = base;

    // Move mesh ribbon positions (update current_position field)
    for (mut ribbon, name) in q_mesh.iter_mut() {
        let bob_pos = if name.as_str() == "bob1_mesh_ribbon" {
            bob1_pos * RENDER_SCALE
        } else {
            (bob1_pos + bob2_pos) * RENDER_SCALE
        };
        ribbon.current_position = bob_pos;
    }
}

fn main() {
    let mut app = App::new();
    
    app
        // Force VSync: cap to the display refresh rate
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoVsync, // classic VSync cap

                    #[cfg(target_arch = "wasm32")]
                    canvas: Some("#bevy".into()),
                    // Keep the WebGL canvas exactly as big as its parent (puts the wasm module in full screen basically)
                    #[cfg(target_arch = "wasm32")]
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
        .insert_resource(ClearColor(bevy::prelude::Color::Srgba(Srgba { red: 84.0 / 255.0, green: 18.0 / 255.0, blue: 18.0 / 255.0, alpha: 1.0 })))
        .add_systems(Startup, setup )
        // Physics on a fixed timestep
        .add_systems(FixedUpdate, step_pendulum)
        // Rendering on the variable-rate Update schedule (interpolation optional)
        .add_systems(Update, draw_pendulum)
        .add_systems(Update, add_ribbon_position);

    #[cfg(feature = "fps_overlay")]
    app.add_plugins(FpsOverlayPlugin::default());

    app.run();
}