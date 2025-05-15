use core::f32;
use std::f32::consts::FRAC_PI_2;

use bevy::{
	color::Color,
	ecs::{component::Component, resource::Resource},
	gizmos::gizmos::Gizmos,
	math::{Isometry2d, Rot2, Vec2, vec2},
	reflect::Reflect,
};

use crate::{
	constants::{GENERAL_EPSILON, PIXEL_EPSILON},
	geom::{circle::Circle, misc::DrawableWithGizmos},
	math::{
		bend_to_abs_angle, between_clockwise, between_counterclockwise,
		clockwise_difference, counterclockwise_difference, midpoint,
	},
};

static ARC_DRAW_SEGMENTS: u32 = 128;

#[derive(Clone, Component, Copy, Debug, Default, Reflect, Resource)]
pub struct Arc {
	/// Angle in radians to middle of arc.
	pub mid: f32,
	/// Angle spanned in radians.
	/// Positive values means endpoint is counter-clockwise [span] radians from startpoint.
	/// Negative values means endpoint is clockwise [span] radians from startpoint.
	pub span: f32,
	/// Radius of underlying circle.
	pub radius: f32,
	/// Center of underlying circle.
	pub center: Vec2,
}

impl DrawableWithGizmos for Arc {
	fn draw_gizmos(&self, gizmos: &mut Gizmos, color: Option<Color>) {
		if self.valid() {
			gizmos
				.arc_2d(
					Isometry2d::new(
						self.center,
						Rot2::radians(self.mid - 0.5 * self.span - FRAC_PI_2),
					),
					self.span,
					self.radius,
					color.unwrap_or(Color::WHITE),
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
				color.unwrap_or(Color::WHITE),
			);
		}
	}
}

impl Arc {
	pub fn from_angles_clockwise(
		start_angle: f32,
		end_angle: f32,
		radius: f32,
		center: Vec2,
	) -> Self {
		let span = -clockwise_difference(start_angle, end_angle);
		let mid = start_angle + 0.5 * span;
		Self { mid, span, radius, center }
	}

	pub fn from_angles_counterclockwise(
		start_angle: f32,
		end_angle: f32,
		radius: f32,
		center: Vec2,
	) -> Self {
		let span = counterclockwise_difference(start_angle, end_angle);
		let mid = start_angle + 0.5 * span;
		Self { mid, span, radius, center }
	}

	pub fn from_bend_and_endpoints(a: Vec2, b: Vec2, bend: f32) -> Self {
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

	pub fn distance_to_point(self, point: Vec2) -> f32 {
		let mut ds = vec![
			point.distance(self.start_point()),
			point.distance(self.end_point()),
		];
		if self.in_span(point) {
			ds.push((point.distance(self.center) - self.radius).abs());
		}
		*ds.iter().min_by(|a, b| a.total_cmp(b)).unwrap()
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

	pub fn params(self) -> [f32; 5] {
		[self.mid, self.span, self.radius, self.center.x, self.center.y]
	}

	pub fn valid(self) -> bool {
		self.params().into_iter().all(f32::is_finite)
			&& self.radius.abs() > PIXEL_EPSILON
			&& self.span.abs() > GENERAL_EPSILON
	}

	pub fn in_span(self, point: Vec2) -> bool {
		let f = if self.span < 0.0 {
			between_clockwise
		} else {
			between_counterclockwise
		};
		f((point - self.center).to_angle(), self.start_angle(), self.end_angle())
	}

	pub fn intersect(self, other: Arc) -> Vec<Vec2> {
		let ps = self.to_circle().intersect(other.to_circle());
		ps.into_iter().filter(|&p| self.in_span(p) && other.in_span(p)).collect()
	}
}
