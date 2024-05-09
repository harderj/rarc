use std::f32::consts::PI;

extern crate derive_more;
use derive_more::Display;

use crate::math::{
	angle_counter_clockwise, circle_center_from_3_points, two_circle_collision,
	Circle, FloatVec2,
};
use bevy::{
	ecs::component::Component, gizmos::gizmos::Gizmos, math::Vec2,
	reflect::Reflect, render::color::Color,
};

#[derive(Clone, Component, Copy, Display, Reflect)]
#[display(fmt = "arc({}, {}, {})", a, b, bend)]
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
		Arc::angle_gen(self.ca(), self.cb(), self.bend)
	}

	pub fn angle_gen(ca: Vec2, cb: Vec2, bend: f32) -> f32 {
		if bend > 0.0 {
			angle_counter_clockwise(ca, cb)
		} else {
			angle_counter_clockwise(cb, ca)
		}
	}

	pub fn ab_average(&self) -> Vec2 {
		0.5 * (self.a + self.b)
	}

	pub fn set_a_keeping_center(&mut self, new_a: Vec2) {
		let c = self.center();
		let alpha = 0.5 * Arc::angle_gen(new_a - c, self.cb(), self.bend);
		let new_bend = 2.0 * (1.0 - f32::cos(alpha)) * self.radius()
			/ (new_a - self.b).length()
			* f32::signum(self.bend);
		self.a = new_a;
		self.bend = new_bend;
		// println!("{}", (self.center() - c).length());
	}

	pub fn set_b_keeping_center(&mut self, new_b: Vec2) -> () {
		let c = self.center();
		let alpha = 0.5 * Arc::angle_gen(self.ca(), new_b - c, self.bend);
		let new_bend = 2.0 * (1.0 - f32::cos(alpha)) * self.radius()
			/ (self.a - new_b).length()
			* f32::signum(self.bend);
		self.b = new_b;
		self.bend = new_bend;
	}

	pub fn angle_a(&self) -> f32 {
		let ca = self.ca();
		f32::atan2(ca.y, ca.x)
	}

	pub fn angle_b(&self) -> f32 {
		let cb = self.cb();
		f32::atan2(cb.y, cb.x)
	}

	pub fn circle(&self) -> Circle {
		FloatVec2 { v: self.center(), f: self.radius() }
	}

	pub fn circle_neg_r(&self) -> Circle {
		FloatVec2 { v: self.center(), f: -self.radius() * f32::signum(self.bend) }
	}

	pub fn collision_idx(&self, other: Arc) -> Option<usize> {
		const TOLERANCE: f32 = 0.001;
		let cols = two_circle_collision(&self.circle(), &other.circle());
		if self.bend < 0.0 && other.bend < 0.0 {
			return Some(1);
		} else if self.bend > 0.0 && other.bend > 0.0 {
			return Some(0);
		} else {
			return None;
		}
	}

	pub fn adjust_to_neighbors(
		&mut self,
		col_idx: (Option<usize>, Option<usize>),
		prev: &Arc,
		next: &Arc,
	) {
		let cols_prev = two_circle_collision(&prev.circle(), &self.circle());
		let cols_next = two_circle_collision(&self.circle(), &next.circle());
		match cols_prev[..] {
			[c] => self.set_a_keeping_center(c),
			[_, _] => {
				col_idx.0.map(|i| self.set_a_keeping_center(cols_prev[i]));
			}
			_ => (), // panic maybe?
		};
		match cols_next[..] {
			[c] => self.set_b_keeping_center(c),
			[_, _] => {
				col_idx.1.map(|i| self.set_b_keeping_center(cols_next[i]));
			}
			_ => (),
		};
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
