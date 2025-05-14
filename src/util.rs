use std::f32::consts::PI;

use bevy::{
	color::Color, ecs::resource::Resource, math::Vec2, reflect::Reflect,
};

use crate::constants::SAME_POINT_TOLERANCE;

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
	Color::hsl(PI * (seed as f32) * 1000.0, 0.7, 0.7)
}

impl Default for FloatResource {
	fn default() -> Self {
		Self { value: Default::default(), scale: 10.0 }
	}
}

pub fn almost_same_point(p: Vec2, q: Vec2) -> bool {
	(p - q).length() < SAME_POINT_TOLERANCE
}
