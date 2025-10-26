use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::time::{Fixed, TimePlugin};
use bevy_vector_shapes::prelude::*;
use std::time::Duration;

use PhyzViz::utils::ODEs;
use PhyzViz::utils::rk4::{self, RK4Prealloc};
use PhyzViz::utils::mesh_ribbon::{spawn_mesh_ribbon, MeshRibbonParams, add_ribbon_position};
use PhyzViz::utils::graph::{spawn_graph_widget, GraphParams, GridlineConfig, draw_graph_widget};
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::{Bloom},
};

#[cfg(feature = "fps_overlay")]
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;

const RENDER_SCALE: f32 = 60.0;

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
    params: DoublePendulum,
    prealloc : RK4Prealloc,
}

// Source : https://web.mit.edu/jorloff/www/chaosTalk/double-pendulum/double-pendulum-en.html
impl ODEs::ODEFunc for DoublePendulum {
    fn call(&self, _t: f32, y: &Vec<f32>, out: &mut Vec<f32>) {
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

        out[0] = dtheta1_dt;
        out[2] = dtheta2_dt;

        let domega1_dt = (
            -g * (2.0 * m1 + m2) * theta1.sin()
            - m2 * g * (theta1 - 2.0 * theta2).sin()
            - 2.0 * m2 * delta.sin()
                * (omega2.powi(2) * l2 + omega1.powi(2) * l1 * delta.cos())
        ) / (l1 * denom);

        out[1] = domega1_dt;

        let domega2_dt = (
            2.0 * delta.sin()
                * (omega1.powi(2) * l1 * (m1 + m2)
                + g * (m1 + m2) * theta1.cos()
                + omega2.powi(2) * l2 * m2 * delta.cos())
        ) / (l2 * denom);

        out[3] = domega2_dt;
    }
}

impl DoublePendulum {
    /// Calculate kinetic energy of the system
    fn kinetic_energy(&self, theta1: f32, omega1: f32, theta2: f32, omega2: f32) -> (f32, f32) {
        let m1 = self.m1;
        let m2 = self.m2;
        let l1 = self.l1;
        let l2 = self.l2;
        
        let delta = theta1 - theta2;
        
        // Kinetic energy formula for double pendulum
        let ke1 = 0.5 * m1 * (l1 * omega1).powi(2);
        let ke2 = 0.5 * m2 * (
            (l1 * omega1).powi(2) + (l2 * omega2).powi(2) 
            + 2.0 * l1 * l2 * omega1 * omega2 * delta.cos()
        );

        (ke1, ke2)
    }
    
    /// Calculate potential energy of the system
    fn potential_energy(&self, theta1: f32, theta2: f32) -> (f32, f32) {
        let m1 = self.m1;
        let m2 = self.m2;
        let l1 = self.l1;
        let l2 = self.l2;
        let g = self.g;
        
        // Taking the pivot as zero potential energy reference
        let h1 = -l1 * theta1.cos();
        let h2 = -l1 * theta1.cos() - l2 * theta2.cos();

        (m1 * g * h1, m2 * g * h2)
    }
}


fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Tonemapping::TonyMcMapface, // 1. Using a tonemapper that desaturates to white is recommended
        Bloom::default(),           // 2. Enable bloom for the camera
        DebandDither::Enabled,      // Optional: bloom causes gradients which cause banding
    ));

    let prealloc = rk4::RK4Prealloc {
        y0: vec![0.0; 4],
        k1: vec![0.0; 4],
        k2: vec![0.0; 4],
        k3: vec![0.0; 4],
        k4: vec![0.0; 4],
        out: vec![0.0; 4],
        func: Box::new(DoublePendulum { m1: 1.0, m2: 1.0, l1: 1.0, l2: 1.0, g: 9.81 }),
    };

    // commands.insert_resource(PendulumState { theta1: 2.899002795870406, omega1: 0.0, theta2: 1.913720799888307, omega2: 0.0, params: DoublePendulum { m1: 1.0, m2: 1.0, l1: 1.0, l2: 1.0, g: 9.81 }, prealloc });
    commands.insert_resource(PendulumState { theta1: 2.0, omega1: 0.0, theta2: 2.0, omega2: 0.0, params: DoublePendulum { m1: 1.0, m2: 1.0, l1: 1.0, l2: 1.0, g: 9.81 }, prealloc });

    // Spawn mesh ribbons (comment out particle ribbons to compare)
    spawn_mesh_ribbon(&mut commands, &mut meshes, &mut materials, "bob1_mesh_ribbon".to_string(), MeshRibbonParams {
        width: 3.0,
        max_points: 1000,
        color: Color::linear_rgba(10.0, 8.7, 10.0, 1.0),
        fade_to_transparent: true,
        ..Default::default()
    });
    spawn_mesh_ribbon(&mut commands, &mut meshes, &mut materials, "bob2_mesh_ribbon".to_string(), MeshRibbonParams {
        width: 3.0,
        max_points: 1000,
        color: Color::linear_rgba(10.0, 8.7, 10.0, 1.0),
        fade_to_transparent: true,
        ..Default::default()
    });

    let mut sprite = Sprite::from_image(asset_server.load("double-pendulum.png"));
    sprite.color = Color::Srgba(Srgba { red: 1.5, green: 1.5, blue: 1.5, alpha: 1.0 });

    // Spawn equations as a sprite in world space
    commands.spawn((
        sprite,
        Transform::from_xyz(0.0, 250.0, -1.0).with_scale(Vec3::splat(0.25)),
    ));

    // Spawn graph widget to track energy or position
    spawn_graph_widget(&mut commands, GraphParams {
        position: Vec2::new(-600.0, 320.0),
        size: Vec2::new(250.0, 150.0),
        max_points: 600,
        line_color: Color::linear_rgba(3.0, 0.6, 0.2, 1.0),
        label: "Bob2 Y-Position".to_string(),
        x_gridlines: GridlineConfig::Fixed { spacing: 2.0 },
        y_gridlines: GridlineConfig::Dynamic {
            min_spacing: 20.0,
            num_lines: 4,
        },
        gridline_origin: Vec2::ZERO,
        ..Default::default()
    });

    // Spawn state space plot (KE vs PE)
    spawn_graph_widget(&mut commands, GraphParams {
        position: Vec2::new(350.0, 320.0),
        size: Vec2::new(250.0, 150.0),
        max_points: 200,
        line_color: Color::linear_rgba(0.2, 3.0, 0.6, 1.0),
        grid_color: Color::srgba(0.5, 0.5, 0.5, 0.3),
        label: "State Space: PE1 vs PE2".to_string(),
        x_gridlines: GridlineConfig::Dynamic {
            min_spacing: 5.0,
            num_lines: 4,
        },
        y_gridlines: GridlineConfig::Dynamic {
            min_spacing: 5.0,
            num_lines: 4,
        },
        gridline_origin: Vec2::ZERO,
        expansion_threshold: 0.15,
        ..Default::default()
    });
}


fn step_pendulum(time_fixed: Res<Time<Fixed>>, mut state: ResMut<PendulumState>) {
    let dt = time_fixed.delta_secs() / 2.0;
    let t = time_fixed.elapsed_secs() / 2.0;

    // let y0 = vec![state.theta1, state.omega1, state.theta2, state.omega2];
    state.prealloc.y0[0] = state.theta1;
    state.prealloc.y0[1] = state.omega1;
    state.prealloc.y0[2] = state.theta2;
    state.prealloc.y0[3] = state.omega2;

    rk4::rk4(t, dt, &mut state.prealloc);
    state.theta1 = state.prealloc.out[0];
    state.omega1 = state.prealloc.out[1];
    state.theta2 = state.prealloc.out[2];
    state.omega2 = state.prealloc.out[3];
}


fn draw_pendulum(
    mut painter: ShapePainter,
    state: Res<PendulumState>,
    mut q_mesh: Query<(&mut PhyzViz::utils::mesh_ribbon::MeshRibbon, &Name)>,
    mut q_graph: Query<&mut PhyzViz::utils::graph::GraphWidget>,
    time_fixed: Res<Time<Fixed>>,
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

    // Update graphs
    let bob2_y = -(bob1_pos.y + bob2_pos.y) * RENDER_SCALE;
    let ke = state.params.kinetic_energy(state.theta1, state.omega1, state.theta2, state.omega2);
    let pe = state.params.potential_energy(state.theta1, state.theta2);

    let mut graph_iter = q_graph.iter_mut();
    
    // First graph: bob2 y-position vs time
    if let Some(mut graph) = graph_iter.next() {
        graph.add_point(time_fixed.elapsed_secs(), bob2_y);
    }
    
    // Second graph: state space (KE vs PE)
    if let Some(mut graph) = graph_iter.next() {
        graph.add_point(pe.0, pe.1);
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
        .add_plugins(Shape2dPlugin::default())
        .insert_resource(ClearColor(bevy::prelude::Color::Srgba(Srgba { red: 84.0 / 255.0, green: 18.0 / 255.0, blue: 18.0 / 255.0, alpha: 1.0 })))
        .add_systems(Startup, setup )
        // Physics on a fixed timestep
        .add_systems(FixedUpdate, step_pendulum)
        // Rendering on the variable-rate Update schedule (interpolation optional)
        .add_systems(Update, draw_pendulum)
        .add_systems(Update, add_ribbon_position)
        .add_systems(Update, draw_graph_widget);

    #[cfg(feature = "fps_overlay")]
    app.add_plugins(FpsOverlayPlugin::default());

    #[cfg(target_arch = "wasm32")]
    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(1.0 / 50.0)));

    #[cfg(not(target_arch = "wasm32"))]
    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(1.0 / 120.0)));

    app.run();
}