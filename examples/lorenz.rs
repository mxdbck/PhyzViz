use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::time::{Fixed, TimePlugin};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::math::vec4;
use bevy_hanabi::prelude::*;
use std::time::Duration;

use PhyzViz::utils::ODEs;
use PhyzViz::utils::rk4;
use PhyzViz::utils::ribbon::{spawn_ribbon_emitter, RibbonParams};
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
};

// Render and ribbon params
const RENDER_SCALE: f32 = 10.0;
const RIBBON_SPAWN_RATE: f32 = 200.0;
const RIBBON_LIFETIME: f32 = 10.0;
const PARTICLE_CAPACITY: u32 = 8_000;
const RIBBON_WIDTH: f32 = 1.0;

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
}

impl ODEs::ODEFunc for Lorenz {
    fn call(&self, _t: f32, y: Vec<f32>) -> Vec<f32> {
        let x = y[0];
        let z = y[2];
        let dy = y[1];

        let dxdt = self.sigma * (dy - x);
        let dydt = x * (self.rho - z) - dy;
        let dzdt = x * dy - self.beta * z;

        vec![dxdt, dydt, dzdt]
    }
}

fn setup(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
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
    });

    // Spawn ribbon for the tracer
    let gradient = bevy_hanabi::Gradient::linear(
        vec4(1.8, 1.4, 3.0, 1.0),
        vec4(1.8, 1.4, 3.0, 0.0),
    );

    spawn_ribbon_emitter(&mut commands, &mut effects, "lorenz_ribbon".to_string(), &RibbonParams {
        width: RIBBON_WIDTH,
        lifetime: RIBBON_LIFETIME,
        capacity: PARTICLE_CAPACITY,
        spawn_rate: RIBBON_SPAWN_RATE,
        color_gradient: gradient,
    });
}

// Integrate Lorenz at a fixed timestep
fn step_lorenz(time_fixed: Res<Time<Fixed>>, mut state: ResMut<LorenzState>) {
    let dt = time_fixed.delta_secs() / 4.0;
    let t = time_fixed.elapsed_secs() / 4.0;

    let y0 = vec![state.x, state.y, state.z];
    let y1 = rk4::rk4(&state.params, t, y0, dt);

    state.x = y1[0];
    state.y = y1[1];
    state.z = y1[2];
}

// Move the ribbon emitter to the current Lorenz position
fn move_ribbon(mut q: Query<&mut Transform, With<ParticleEffect>>, state: Res<LorenzState>) {
    if let Ok(mut transform) = q.single_mut() {
        transform.translation = Vec3::new(state.x, state.y, state.z) * RENDER_SCALE;
    }
}

fn main() {
    App::new()
        .add_plugins(
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
        .add_plugins(HanabiPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, step_lorenz)
        .add_systems(Update, move_ribbon)
        .run();
}
