use std::f32::consts::{E, PI};

use bevy::{
	color::Color, ecs::resource::Resource, math::Vec2, reflect::Reflect,
};

use crate::constants::GENERAL_EPSILON;

#[derive(Copy, Clone, Reflect, Resource)]
pub struct FloatResource {
	pub value: f32,
	pub scale: f32,
}

impl FloatResource {
	pub fn get(self) -> f32 {
		self.value * self.scale
	}
}

pub fn color_hash(seed: usize) -> Color {
	Color::hsl((E * PI * (seed as f32) * 1000.0) % 360.0, 0.8, 0.4)
}

impl Default for FloatResource {
	fn default() -> Self {
		Self { value: Default::default(), scale: 10.0 }
	}
}

pub fn almost_same_point(p: Vec2, q: Vec2) -> bool {
	(p - q).length() < GENERAL_EPSILON
}
