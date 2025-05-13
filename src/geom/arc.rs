use std::{f32::consts::FRAC_PI_2, vec};

use bevy::{
	color::Color,
	ecs::{component::Component, resource::Resource, world::FromWorld},
	gizmos::gizmos::Gizmos,
	math::{Isometry2d, Rot2, Vec2, vec2},
	reflect::Reflect,
};

use crate::{
	constants::{GENERAL_EPSILON, PIXEL_EPSILON},
	geom::{circle::Circle, misc::DrawableWithGizmos},
	math::{bend_to_abs_angle, midpoint},
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

impl DrawableWithGizmos for Arc {
	fn draw_gizmos(&self, gizmos: &mut Gizmos, color: Color) {
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
			let m = self.mid_arc_point();
			let angle = (self.end_point() - self.start_point()).to_angle();
			gizmos.linestrip_2d(
				[
					m + vec2(5.0, 5.0).rotate(Vec2::from_angle(angle)),
					m,
					m + vec2(5.0, -5.0).rotate(Vec2::from_angle(angle)),
				],
				color,
			);
		}
	}
}

impl Arc {
	pub fn with_radius(self, radius: f32) -> Self {
		let mut copy = self;
		copy.radius = radius;
		copy
	}

	pub fn with_span(self, span: f32) -> Self {
		let mut copy = self;
		copy.span = span;
		copy
	}

	pub fn start_angle(self) -> f32 {
		self.mid - 0.5 * self.span
	}

	pub fn end_angle(self) -> f32 {
		self.mid + 0.5 * self.span
	}

	pub fn start_point(self) -> Vec2 {
		self.center + Vec2::from_angle(self.start_angle()) * self.radius
	}

	pub fn end_point(self) -> Vec2 {
		self.center + Vec2::from_angle(self.end_angle()) * self.radius
	}

	pub fn mid_arc_point(self) -> Vec2 {
		self.center + Vec2::from_angle(self.mid) * self.radius
	}

	pub fn minkowski_disc(self, radius: f32) -> Vec<Box<dyn DrawableWithGizmos>> {
		// consider to make this cleaner by changing circles into arcs
		return vec![
			Box::new(self.with_radius(self.radius + radius)),
			Box::new(self.with_radius(self.radius - radius).with_span(-self.span)),
			Box::new(Circle { radius, center: self.start_point() }),
			Box::new(Circle { radius, center: self.end_point() }),
		];
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
		let Circle { radius: _, center } = Circle::from_3_points(a, b, arc_mid);
		let span = bend_to_abs_angle(bend);
		let mid = (arc_mid - center).to_angle();
		Self { mid, span, radius, center }
	}

	pub fn to_circle(self) -> Circle {
		Circle { radius: self.radius, center: self.center }
	}
}
