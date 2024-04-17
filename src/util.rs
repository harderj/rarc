use bevy::{ecs::resource::Resource, math::Vec2, reflect::Reflect};

use crate::constants::SAME_POINT_TOLERANCE;

#[derive(Reflect, Resource)]
pub struct TimeResource {
	pub time: f32,
	pub speed: f32,
}

impl Default for TimeResource {
	fn default() -> Self {
		Self { time: Default::default(), speed: 10.0 }
	}
}

pub fn almost_same_point(p: Vec2, q: Vec2) -> bool {
	(p - q).length() < SAME_POINT_TOLERANCE
}
