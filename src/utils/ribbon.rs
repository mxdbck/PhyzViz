use bevy::prelude::*;
use bevy::math::vec4;
use bevy_hanabi::prelude::*;

use crate::utils::ODEs;
use crate::utils::rk4;
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::{Bloom},
};

pub struct RibbonParams {
    pub width: f32,
    pub lifetime: f32,
    pub capacity: u32,
    pub spawn_rate: f32,
    pub color_gradient: bevy_hanabi::Gradient<Vec4>,
}

impl Default for RibbonParams {
    fn default() -> Self {
        Self {
            width: 5.0,
            lifetime: 1.5,
            capacity: 100,
            spawn_rate: 60.0,
            color_gradient: bevy_hanabi::Gradient::linear(
        vec4(3.0, 0.0, 0.0, 1.0),
            vec4(3.0, 0.0, 0.0, 0.0),
            ),
        }
    }
}

// Reusable: spawns a ribbon particle emitter with the given name, width, lifetime and color gradient
pub fn spawn_ribbon_emitter(
    commands: &mut Commands,
    effects: &mut Assets<EffectAsset>,
    name: String,
    params: &RibbonParams,
) -> Entity {
    let writer = ExprWriter::new();
    let init_position_attr = SetAttributeModifier {
        attribute: Attribute::POSITION,
        value: writer.lit(Vec3::ZERO).expr(),
    };
    let init_age_attr = SetAttributeModifier {
        attribute: Attribute::AGE,
        value: writer.lit(0.0).expr(),
    };
    let init_lifetime_attr = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: writer.lit(params.lifetime).expr(),
    };
    let init_size_attr = SetAttributeModifier {
        attribute: Attribute::SIZE,
        value: writer.lit(params.width).expr(),
    };
    let init_ribbon_id = SetAttributeModifier {
        attribute: Attribute::RIBBON_ID,
        value: writer.lit(0u32).expr(),
    };
    let spawner = SpawnerSettings::rate(params.spawn_rate.into());
    let effect = EffectAsset::new(params.capacity, spawner, writer.finish())
        .with_name(name.clone())
        .with_motion_integration(MotionIntegration::None)
        .with_simulation_space(SimulationSpace::Global)
        .init(init_position_attr)
        .init(init_age_attr)
        .init(init_lifetime_attr)
        .init(init_size_attr)
        .init(init_ribbon_id)
        .render(SizeOverLifetimeModifier {
            gradient: bevy_hanabi::Gradient::linear(Vec3::splat(params.width), Vec3::splat(0.0)),
            ..default()
        })
        .render(ColorOverLifetimeModifier::new(params.color_gradient.clone()));
    let handle = effects.add(effect);
    commands
        .spawn((ParticleEffect::new(handle), Transform::default(), Name::new(name)))
        .id()
}