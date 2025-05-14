use core::f32;
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
	color::Color,
	ecs::{component::Component, resource::Resource, world::FromWorld},
	gizmos::gizmos::Gizmos,
	math::{Isometry2d, Rot2, Vec2, vec2},
	reflect::Reflect,
};
use petgraph::graph::UnGraph;

use crate::{
	constants::{GENERAL_EPSILON, PIXEL_EPSILON},
	geom::{circle::Circle, misc::DrawableWithGizmos},
	math::{bend_to_abs_angle, counterclockwise_difference, midpoint},
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
					m + vec2(-5.0, 5.0).rotate(Vec2::from_angle(angle)),
					m,
					m + vec2(-5.0, -5.0).rotate(Vec2::from_angle(angle)),
				],
				color,
			);
		}
	}
}

impl Arc {
	pub fn from_angles(
		start_angle: f32,
		end_angle: f32,
		radius: f32,
		center: Vec2,
	) -> Self {
		let span = counterclockwise_difference(start_angle, end_angle);
		let mid = start_angle + 0.5 * span;
		Self { mid, span, radius, center }
	}

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

	pub fn minkowski_disc(self, radius: f32) -> UnGraph<Arc, Vec2> {
		// consider to make this cleaner by changing circles into arcs
		let mut g = UnGraph::<Arc, Vec2>::new_undirected();
		let _idx1 = g.add_node(self.with_radius(self.radius + radius));
		if radius.abs() < self.radius.abs() {
			let _idx2 = g.add_node(Arc {
				radius,
				center: self.end_point(),
				mid: self.end_angle() + FRAC_PI_2,
				span: PI,
			});
			let _idx3 = g
				.add_node(self.with_radius(self.radius - radius).with_span(-self.span));
			let _idx4 = g.add_node(Arc {
				radius,
				center: self.start_point(),
				mid: self.start_angle() - FRAC_PI_2,
				span: PI,
			});
		} else {
			if let Some(&intersection) = Circle::new(radius, self.start_point())
				.intersect_circle(Circle::new(radius, self.end_point()))
				.get(1)
			{
				let _idx2 = g.add_node(Arc::from_angles(
					self.end_angle(),
					(intersection - self.end_point()).to_angle(),
					radius,
					self.end_point(),
				));
				let _idx3 = g.add_node(Arc::from_angles(
					(intersection - self.start_point()).to_angle(),
					self.start_angle(),
					radius,
					self.start_point(),
				));
			}
		}
		g
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
