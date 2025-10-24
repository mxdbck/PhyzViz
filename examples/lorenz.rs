use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::time::{Fixed, TimePlugin};
#[cfg(feature = "fps_overlay")]
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use std::time::Duration;

use PhyzViz::utils::ODEs;
use PhyzViz::utils::rk4;
use PhyzViz::utils::mesh_ribbon::{spawn_mesh_ribbon, MeshRibbonParams, add_ribbon_position};
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
};

// Render and ribbon params
const RENDER_SCALE: f32 = 10.0;
const RIBBON_WIDTH: f32 = 5.0;
const RIBBON_MAX_POINTS: usize = 20000;

// Lorenz system
pub struct Lorenz {
    pub sigma: f32,
    pub rho: f32,
    pub beta: f32,
}

#[derive(Resource)]
struct LorenzState {
    x: f32,
    y: f32,
    z: f32,
    params: Lorenz,
    prealloc: rk4::RK4Prealloc,
}

impl ODEs::ODEFunc for Lorenz {
    fn call(&self, _t: f32, y: &Vec<f32>, out: &mut Vec<f32>) {
        let x = y[0];
        let z = y[2];
        let dy = y[1];

        let dxdt = self.sigma * (dy - x);
        let dydt = x * (self.rho - z) - dy;
        let dzdt = x * dy - self.beta * z;

        out[0] = dxdt;
        out[1] = dydt;
        out[2] = dzdt;
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
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

    let prealloc = rk4::RK4Prealloc {
        y0: vec![0.0; 3],
        k1: vec![0.0; 3],
        k2: vec![0.0; 3],
        k3: vec![0.0; 3],
        k4: vec![0.0; 3],
        out: vec![0.0; 3],
        func: Box::new(Lorenz {
            sigma: 10.0,
            rho: 28.0,
            beta: 8.0 / 3.0,
        }),
    };

    // Lorenz initial state
    commands.insert_resource(LorenzState {
        x: 10.0,
        y: 10.0,
        z: 10.0,
        params: Lorenz {
            sigma: 10.0,
            rho: 28.0,
            beta: 8.0 / 3.0,
        },
        prealloc,
    });

    let scale = 2.0;

    // Spawn mesh ribbon for the tracer
    spawn_mesh_ribbon(
        &mut commands,
        &mut meshes,
        &mut materials,
        "lorenz_ribbon".to_string(),
        MeshRibbonParams {
            width: RIBBON_WIDTH,
            max_points: RIBBON_MAX_POINTS,
            color: Color::linear_rgba(scale * 1.8, scale * 1.4, scale * 3.0, 1.0),
            fade_to_transparent: true,
            width_variation: PhyzViz::utils::mesh_ribbon::InterpolationType::Poly(0.2),
            transparency_variance: PhyzViz::utils::mesh_ribbon::InterpolationType::Poly(0.2),
        }
    );
}

// Integrate Lorenz at a fixed timestep
fn step_lorenz(time_fixed: Res<Time<Fixed>>, mut state: ResMut<LorenzState>) {
    let dt = time_fixed.delta_secs() / 4.0;
    let t = time_fixed.elapsed_secs() / 4.0;

    state.prealloc.y0[0] = state.x;
    state.prealloc.y0[1] = state.y;
    state.prealloc.y0[2] = state.z;
    rk4::rk4(t, dt, &mut state.prealloc);

    state.x = state.prealloc.out[0];
    state.y = state.prealloc.out[1];
    state.z = state.prealloc.out[2];
}

// Update the ribbon position to the current Lorenz position
fn update_ribbon(mut q_mesh: Query<&mut PhyzViz::utils::mesh_ribbon::MeshRibbon>, state: Res<LorenzState>) {
    if let Ok(mut ribbon) = q_mesh.single_mut() {
        let pos = Vec3::new(state.x, state.y, state.z) * RENDER_SCALE;
        ribbon.current_position = pos;
    }
}

fn main() {

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::Fifo,
                canvas: Some("#bevy".into()),
                    fit_canvas_to_parent: true,
                    resizable: true,
                    ..default()
                }),
                ..default()
            })
            .set(TimePlugin::default()),
        )
        // Fixed step (e.g., 120 Hz)
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(1.0 / 120.0)))
        // .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, step_lorenz)
        .add_systems(Update, update_ribbon)
        .add_systems(Update, add_ribbon_position);

    #[cfg(feature = "fps_overlay")]
    app.add_plugins(FrameTimeDiagnosticsPlugin::default());

    app.run();

}
