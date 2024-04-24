use std::f32::consts::PI;

use crate::math::{
	angle_counter_clockwise, circle_center_from_3_points, two_circle_collision,
	Circle, FloatVec2,
};
use bevy::{
	ecs::component::Component, gizmos::gizmos::Gizmos, math::Vec2,
	reflect::Reflect, render::color::Color,
};

#[derive(Component, Reflect, Clone)]
pub struct Arc {
	pub a: Vec2,
	pub b: Vec2,
	pub bend: f32, // arc height (positive is right of A->B)
}

impl Arc {
	pub fn ab(&self) -> Vec2 {
		self.b - self.a
	}

	pub fn outward(&self) -> Vec2 {
		self.ab().rotate(Vec2::NEG_Y)
	}

	pub fn extreme(&self) -> Vec2 {
		0.5 * (self.a + self.b) + 0.5 * self.outward() * self.bend
	}

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
		if self.bend > 0.0 {
			angle_counter_clockwise(self.ca(), self.cb())
		} else {
			angle_counter_clockwise(self.cb(), self.ca())
		}
	}

	pub fn set_a_keeping_center(&mut self, new_a: Vec2) {
		let c = self.center();
		let ca = new_a - c;
		assert!(ca.length() - self.radius() < 0.001);
		let alpha = if self.bend > 0.0 {
			angle_counter_clockwise(new_a - c, self.cb()) / 2.0
		} else {
			angle_counter_clockwise(self.cb(), new_a - c) / 2.0
		};
		self.a = new_a;
		self.bend = (1.0 - f32::cos(alpha)) * f32::signum(self.bend);
		assert!((self.center() - c).length() < 0.001);
	}

	pub fn set_b_keeping_center(&mut self, new_b: Vec2) {
		let c = self.center();
		let cb = new_b - c;
		assert!(cb.length() - self.radius() < 0.001);
		let alpha = if self.bend > 0.0 {
			angle_counter_clockwise(self.ca(), cb) / 2.0
		} else {
			angle_counter_clockwise(cb, self.ca()) / 2.0
		};
		self.b = new_b;
		self.bend = (1.0 - f32::cos(alpha)) * f32::signum(self.bend);
		println!("{}", (self.center() - c).length());
		assert!((self.center() - c).length() < 0.001);
		// let alpha = angle_counter_clockwise(self.ca(), new_b - c) / 2.0;
		// let r = self.radius();
		// let ca = self.a - c;
		// let cb = self.b - c;
		// let alpha = (2.0 * PI + (f32::atan2(cb.y, cb.x) - f32::atan2(ca.y, ca.x))) % (2.0 * PI);
	}

	pub fn angle_a(&self) -> f32 {
		let ca = self.ca();
		f32::atan2(ca.y, ca.x)
	}

	pub fn angle_b(&self) -> f32 {
		let cb = self.cb();
		f32::atan2(cb.y, cb.x)
	}

	fn circle(&self) -> Circle {
		FloatVec2 { v: self.center(), f: self.radius() }
	}

	pub fn collision_idx(&self, other: Arc) -> Option<usize> {
		const TOLERANCE: f32 = 0.001;
		let cols = two_circle_collision(&self.circle(), &other.circle());
		if cols.len() < 2 {
			None
		} else {
			let b_dist_0 = (cols[0] - self.b).length();
			let b_dist_1 = (cols[1] - self.b).length();
			if b_dist_0 < TOLERANCE {
				Some(0)
			} else if b_dist_1 < TOLERANCE {
				Some(1)
			} else {
				None
			}
		}
	}

	pub fn adjust_b(&mut self, next: Arc, col_idx: usize) -> Option<Vec2> {
		let cols_next = two_circle_collision(&self.circle(), &next.circle());
		if cols_next.len() > 1 {
			let col = cols_next[col_idx];
			self.set_b_keeping_center(col);
			Some(col)
		} else {
			None
		}
	}

	pub fn shrink_keeping_center(&mut self, amount: f32) {
		let r = self.radius();
		let c = self.center();
		let ang_a = self.angle_a();
		let ang_b = self.angle_b();
		let new_r = r - amount * f32::signum(self.bend);
		let new_a = c + new_r * Vec2::new(f32::cos(ang_a), f32::sin(ang_a));
		let new_b = c + new_r * Vec2::new(f32::cos(ang_b), f32::sin(ang_b));
		self.a = new_a;
		self.b = new_b;
	}

	pub fn draw(&self, gizmos: &mut Gizmos, color: Color) {
		gizmos.circle_2d(Vec2::from_array(self.a.into()), 2.0, Color::BLACK);
		gizmos.circle_2d(Vec2::from_array(self.b.into()), 4.0, Color::GRAY);
		gizmos.arc_2d(
			Vec2::from_array(self.center().into()),
			self.outward().angle_between(Vec2::Y)
				+ (self.bend < 0.0).then_some(PI).unwrap_or(0.0),
			self.angle(),
			self.radius(),
			color,
		);
	}
}
