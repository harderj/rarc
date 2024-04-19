use std::f32::consts::PI;

use crate::math::circle_center_from_3_points;
use bevy::{
	ecs::component::Component, gizmos::gizmos::Gizmos, math::{Mat2, Vec2},
	reflect::Reflect, render::color::Color,
};

#[derive(Component, Reflect)]
pub struct Arc {
	pub a: Vec2,
	pub b: Vec2,
	pub s: f32, // arc height (positive is right of A->B)
}

#[allow(dead_code)]
impl Arc {
	pub fn ab(&self) -> Vec2 { self.b - self.a }

	pub fn sv(&self) -> Vec2 {
		self.ab().rotate(Vec2::NEG_Y).normalize() * self.s
	}

	pub fn extreme(&self) -> Vec2 { self.sv() + 0.5 * (self.a + self.b) }

	pub fn center(&self) -> Vec2 {
		circle_center_from_3_points(self.a, self.b, self.extreme())
	}

	pub fn ca(&self) -> Vec2 {
		self.a - self.center()
	}

	pub fn cb(&self) -> Vec2 {
		self.b - self.center()
	}

	pub fn radius(&self) -> f32 {
		self.ca().length()
	}

	pub fn angle(&self) -> f32 {
		let mut r = Mat2::from_cols(self.ca(), self.cb())
			.determinant()
			.atan2(self.ca().dot(self.cb())) * f32::signum(self.s);
		if r < 0.0 { r += 2.0 * PI }
		r
	}

	pub fn angle_a(&self) -> f32 {
		self.ca().angle_between(Vec2::new(1.0, 0.0))
	}
	
	pub fn angle_b(&self) -> f32 {
		self.cb().angle_between(Vec2::new(1.0, 0.0))
	}

	pub fn draw(&self, gizmos: &mut Gizmos) {
		gizmos.circle_2d(Vec2::from_array(self.a.into()), 2.0, Color::GRAY);
		gizmos.circle_2d(
			Vec2::from_array(self.b.into()),
			4.0,
			Color::DARK_GRAY,
		);
		gizmos.circle_2d(
			Vec2::from_array(self.center().into()),
			6.0,
			Color::BLUE,
		);
		gizmos.circle_2d(
			Vec2::from_array(self.extreme().into()),
			6.0,
			Color::YELLOW,
		);
		gizmos.arc_2d(
			Vec2::from_array(self.center().into()),
			self.sv().angle_between(Vec2::Y),
			self.angle(),
			self.radius(),
			Color::RED,
		);
	}
}
