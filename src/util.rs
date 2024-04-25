use bevy::{
	ecs::system::Resource, gizmos::gizmos::Gizmos, reflect::Reflect,
	render::color::Color,
};

use crate::math::FloatVec2;

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

pub fn gizmo_circle(gizmos: &mut Gizmos, circle: FloatVec2, color: Color) {
	gizmos.circle_2d(circle.v, circle.f, color);
}
