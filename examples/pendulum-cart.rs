use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::time::{Fixed, TimePlugin};
use bevy_vector_shapes::prelude::*;
use std::time::Duration;

use rapier2d_f64::prelude::*;

use PhyzViz::utils::mesh_ribbon::{spawn_mesh_ribbon, MeshRibbonParams, add_ribbon_position};
use PhyzViz::utils::graph::{spawn_graph_widget, GraphParams, GridlineConfig, draw_graph_widget};
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
};

#[cfg(feature = "fps_overlay")]
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;

const RENDER_SCALE: f32 = 60.0;

const INTEGRATION_TIME_STEP: f64 = 1.0 / 480.0;

#[cfg(not(target_arch = "wasm32"))]
const BEVY_FIXED_TIME_STEP: f64 = 1.0 / 120.0;

#[cfg(target_arch = "wasm32")]
const BEVY_FIXED_TIME_STEP: f64 = 1.0 / 50.0;

// Physics parameters
const CART_MASS: f64 = 2.0;
const PENDULUM_MASS: f64 = 1.0;
const PENDULUM_LENGTH: f64 = 2.0;
const GRAVITY: f64 = 9.81;
const INITIAL_ANGLE: f64 = 11.0 * std::f64::consts::PI / 12.0; // Initial angle in radians (0 = hanging down, positive = right)

#[derive(Resource)]
struct PhysicsWorld {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
    cart_handle: RigidBodyHandle,
    pendulum_handle: RigidBodyHandle,
}

impl PhysicsWorld {
    fn new() -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        let mut impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();

        // Create the cart (can only move horizontally)
        let cart_body = RigidBodyBuilder::dynamic()
            .translation(vector![0.0, 0.0])
            .linear_damping(0.0)
            .locked_axes(LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y)
            .build();
        let cart_handle = rigid_body_set.insert(cart_body);

        let cart_collider = ColliderBuilder::cuboid(0.3, 0.2)
            .density(CART_MASS / (0.6 * 0.4))
            .build();
        collider_set.insert_with_parent(cart_collider, cart_handle, &mut rigid_body_set);

        // Create the pendulum bob at the initial angle position
        // Position relative to cart: (L*sin(θ), -L*cos(θ))
        let initial_x = PENDULUM_LENGTH * INITIAL_ANGLE.sin();
        let initial_y = -PENDULUM_LENGTH * INITIAL_ANGLE.cos();
        
        let pendulum_body = RigidBodyBuilder::dynamic()
            .translation(vector![initial_x, initial_y])
            .build();
        let pendulum_handle = rigid_body_set.insert(pendulum_body);

        let pendulum_collider = ColliderBuilder::ball(0.12)
            .density(PENDULUM_MASS / (std::f64::consts::PI * 0.12 * 0.12))
            .build();
        collider_set.insert_with_parent(pendulum_collider, pendulum_handle, &mut rigid_body_set);

        // Create revolute joint between cart and pendulum
        // The joint anchor in the pendulum's local frame needs to account for the initial angle
        let joint = RevoluteJointBuilder::new()
            .local_anchor1(point![0.0, 0.0])
            .local_anchor2(point![-initial_x, -initial_y]);
        impulse_joint_set.insert(cart_handle, pendulum_handle, joint, true);

        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = INTEGRATION_TIME_STEP;
        integration_parameters.num_solver_iterations = 16;
        integration_parameters.normalized_allowed_linear_error = 1.0e-5;

        Self {
            rigid_body_set,
            collider_set,
            impulse_joint_set,
            multibody_joint_set,
            integration_parameters,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            ccd_solver: CCDSolver::new(),
            cart_handle,
            pendulum_handle,
        }
    }

    fn step(&mut self) {
        let gravity = vector![0.0, -GRAVITY];
        let physics_hooks = ();
        let event_handler = ();

        for _ in 0..(BEVY_FIXED_TIME_STEP / INTEGRATION_TIME_STEP) as usize {
        self.physics_pipeline.step(
            &gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &physics_hooks,
            &event_handler,
        );}
    }

    fn cart_position(&self) -> Vector<f64> {
        self.rigid_body_set[self.cart_handle].translation().clone()
    }

    fn cart_velocity(&self) -> Vector<f64> {
        self.rigid_body_set[self.cart_handle].linvel().clone()
    }

    fn pendulum_position(&self) -> Vector<f64> {
        self.rigid_body_set[self.pendulum_handle].translation().clone()
    }

    fn pendulum_velocity(&self) -> Vector<f64> {
        self.rigid_body_set[self.pendulum_handle].linvel().clone()
    }

    fn total_energy(&self) -> (f64, f64) {
        let cart = &self.rigid_body_set[self.cart_handle];
        let pendulum = &self.rigid_body_set[self.pendulum_handle];

        // Kinetic energy
        let cart_ke = 0.5 * CART_MASS * cart.linvel().norm_squared();
        let pendulum_ke = 0.5 * PENDULUM_MASS * pendulum.linvel().norm_squared();
        let total_ke = cart_ke + pendulum_ke;

        // Potential energy (taking cart level as zero reference)
        let cart_pe = 0.0;
        let pendulum_pe = PENDULUM_MASS * GRAVITY * pendulum.translation().y;
        let total_pe = cart_pe + pendulum_pe;

        (total_ke, total_pe)
    }

    fn pendulum_angle(&self) -> f64 {
        let cart_pos = self.cart_position();
        let pendulum_pos = self.pendulum_position();
        
        // Vector from cart to pendulum
        let dx = pendulum_pos.x - cart_pos.x;
        let dy = pendulum_pos.y - cart_pos.y;
        
        // Angle from vertical (pointing up is 0, increases clockwise)
        // atan2(dx, -dy) gives angle from up
        let angle = dx.atan2(dy);
        
        angle
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Tonemapping::TonyMcMapface,
        Bloom::default(),
        DebandDither::Enabled,
    ));

    // Initialize physics world (pendulum already at initial angle)
    let physics = PhysicsWorld::new();
    
    commands.insert_resource(physics);

    // Spawn mesh ribbon for pendulum trail
    spawn_mesh_ribbon(
        &mut commands,
        &mut meshes,
        &mut materials,
        "pendulum_trail".to_string(),
        MeshRibbonParams {
            width: 10.0,
            max_points: 200,
            color: Color::linear_rgba(0.4, 1.362, 1.995, 1.0), // Lighter than clear color
            fade_to_transparent: true,
            width_variation: PhyzViz::utils::mesh_ribbon::InterpolationType::Poly(10.0),
            ..Default::default()
        },
    );
    
    // Graph for cart position
    spawn_graph_widget(&mut commands, GraphParams {
        position: Vec2::new(-600.0, 320.0),
        size: Vec2::new(250.0, 150.0),
        max_points: 600,
        line_color: Color::linear_rgba(0.2, 0.6, 3.0, 1.0),
        label: "Cart X-Position".to_string(),
        x_gridlines: GridlineConfig::Fixed { spacing: 4.0 },
        y_gridlines: GridlineConfig::Dynamic {
            min_spacing: 20.0,
            num_lines: 4,
        },
        gridline_origin: Vec2::ZERO,
        show_current_x: false,
        show_current_y: true,
        font_size: 14.0,
        ..Default::default()
    });

    // Graph for pendulum angle vs time
    spawn_graph_widget(&mut commands, GraphParams {
        position: Vec2::new(350.0, 320.0),
        size: Vec2::new(250.0, 150.0),
        max_points: 600,
        line_color: Color::linear_rgba(3.0, 0.6, 0.2, 1.0),
        grid_color: Color::srgba(0.5, 0.5, 0.5, 0.3),
        label: "Pendulum Angle".to_string(),
        x_gridlines: GridlineConfig::Fixed { spacing: 4.0 },
        y_gridlines: GridlineConfig::Dynamic {
            min_spacing: 20.0,
            num_lines: 4,
        },
        gridline_origin: Vec2::ZERO,
        expansion_threshold: 0.15,
        show_current_x: false,
        show_current_y: true,
        font_size: 14.0,
        ..Default::default()
    });
}

fn step_physics(mut physics: ResMut<PhysicsWorld>) {
    physics.step();
}

fn draw_system(
    mut painter: ShapePainter,
    physics: Res<PhysicsWorld>,
    mut q_mesh: Query<&mut PhyzViz::utils::mesh_ribbon::MeshRibbon>,
    mut q_graph: Query<&mut PhyzViz::utils::graph::GraphWidget>,
    time_fixed: Res<Time<Fixed>>,
) {
    painter.scale(Vec3::splat(RENDER_SCALE));

    let cart_pos = physics.cart_position();
    let pendulum_pos = physics.pendulum_position();

    let base = painter.transform;

    // Draw rail
    painter.transform = base;
    painter.thickness = 0.03;
    painter.set_color(Srgba {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        alpha: 0.1,
    });
    painter.line(Vec3::new(-8.0, 0.0, -0.1), Vec3::new(8.0, 0.0, -0.1));

    // Draw cart
    let cart_render_pos = Vec3::new(cart_pos.x as f32, cart_pos.y as f32, 0.0);
    
    painter.transform = base;
    painter.thickness = 0.05;
    painter.set_color(Srgba {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        alpha: 0.8,
    });
    painter.translate(cart_render_pos);
    painter.rect(Vec2::new(0.6, 0.4));

    // Draw pendulum rod
    let pendulum_render_pos = Vec3::new(pendulum_pos.x as f32, pendulum_pos.y as f32, 0.0);
    
    painter.transform = base;
    painter.thickness = 0.03;
    painter.set_color(Srgba {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        alpha: 0.8,
    });
    painter.line(cart_render_pos, pendulum_render_pos);

    // Draw cart pivot
    let mut t = base;
    t.translation.z += 0.001;
    painter.transform = t;
    painter.thickness = 0.02;
    painter.hollow = false;
    painter.set_color(Srgba {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        alpha: 1.0,
    });
    painter.translate(cart_render_pos);
    painter.circle(0.07);

    // Draw pendulum bob
    let mut t2 = base;
    t2.translation.z += 0.002;
    painter.transform = t2;
    painter.translate(pendulum_render_pos);
    painter.set_color(Color::linear_rgba(2.0 * 0.2, 2.0 * 0.681, 2.0 * 0.999, 1.0)); // Brighter for bloom
    painter.circle(0.12);

    painter.transform = base;

    // Update mesh ribbon
    if let Ok(mut ribbon) = q_mesh.single_mut() {
        ribbon.current_position = pendulum_render_pos * RENDER_SCALE;
    }

    // Update graphs
    let angle = physics.pendulum_angle();
    let mut graph_iter = q_graph.iter_mut();

    // First graph: cart x-position vs time
    if let Some(mut graph) = graph_iter.next() {
        graph.add_point(time_fixed.elapsed_secs(), cart_pos.x as f32 * RENDER_SCALE);
    }

    // Second graph: pendulum angle vs time
    if let Some(mut graph) = graph_iter.next() {
        graph.add_point(time_fixed.elapsed_secs(), angle as f32 * RENDER_SCALE);
    }
}

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
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
    .add_plugins(Shape2dPlugin::default())
    .insert_resource(ClearColor(bevy::prelude::Color::Srgba(Srgba {
        red: 0.067,
        green: 0.227,
        blue: 0.333,
        alpha: 1.0,
    })))
    .add_systems(Startup, setup)
    .add_systems(FixedUpdate, step_physics)
    .add_systems(Update, draw_system)
    .add_systems(Update, add_ribbon_position)
    .add_systems(Update, draw_graph_widget);

    #[cfg(feature = "fps_overlay")]
    app.add_plugins(FpsOverlayPlugin::default());

    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(
        BEVY_FIXED_TIME_STEP,
    )));

    app.run();
}