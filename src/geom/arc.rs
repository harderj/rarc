use std::f32::consts::FRAC_PI_2;

use bevy::{
	color::Color,
	ecs::{component::Component, resource::Resource, world::FromWorld},
	gizmos::gizmos::Gizmos,
	math::{Isometry2d, Rot2, Vec2},
	reflect::Reflect,
};

use crate::{
	constants::{GENERAL_EPSILON, PIXEL_EPSILON},
	math::{
		Circle, FloatVec2, bend_to_abs_angle, circle_center_from_3_points, midpoint,
	},
};

static ARC_DRAW_SEGMENTS: u32 = 128;

#[derive(Clone, Component, Copy, Debug, Reflect, Resource, FromWorld)]
pub struct Arc {
	pub mid: f32,
	pub span: f32,
	pub radius: f32,
	/// Center of circle
	pub center: Vec2,
}

impl Arc {
	pub fn draw(self, gizmos: &mut Gizmos, color: Color) {
		if self.valid() {
			gizmos
				.arc_2d(
					Isometry2d::new(
						self.center,
						Rot2::radians(self.mid - 0.5 * self.span - FRAC_PI_2),
					),
					self.span,
					self.radius,
					color,
				)
				.resolution(ARC_DRAW_SEGMENTS);
		}
	}

	pub fn params(self) -> [f32; 5] {
		[self.mid, self.span, self.radius, self.center.x, self.center.y]
	}

	pub fn valid(self) -> bool {
		self.params().into_iter().all(f32::is_finite)
			&& self.radius.abs() > PIXEL_EPSILON
			&& self.span.abs() > GENERAL_EPSILON
	}

	pub fn from_a_b_bend(a: Vec2, b: Vec2, bend: f32) -> Self {
		let ab = b - a;
		let perp = ab.normalize().rotate(Vec2::Y);
		let radius =
			ab.length() / (2.0 * f32::sqrt((2.0 - bend.abs()) * bend.abs()));
		let arc_mid = midpoint(a, b) + perp * bend * radius;
		let center = circle_center_from_3_points(a, b, arc_mid);
		let span = bend_to_abs_angle(bend);
		let mid = (arc_mid - center).to_angle();
		Self { mid, span, radius, center }
	}

	pub fn circle(self) -> Circle {
		FloatVec2(self.radius, self.center)
	}
}
