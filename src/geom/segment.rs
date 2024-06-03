extern crate derive_more;
use std::f32::consts::PI;

use derive_more::Display;

use bevy::{
	ecs::component::Component, gizmos::gizmos::Gizmos, math::Vec2,
	reflect::Reflect, render::color::Color,
};

use crate::math::{angle_counter_clockwise, bool_to_sign, Circle, FloatVec2};

#[derive(Clone, Copy, Display, Reflect, PartialEq)]
pub enum Bend {
	Inward,
	Outward,
}

#[derive(Component, Copy, Reflect, Clone, Display)]
#[display(fmt = "segment({}, {})", initial, bend)]
pub struct Segment {
	pub initial: Vec2,
	pub center: Vec2,
	pub bend: Bend,
}

#[derive(Display)]
#[display(fmt = "collision({}, {})", kind, time_place)]
pub struct Collision {
	pub time_place: FloatVec2,
	pub kind: CollisionType,
}

#[derive(Display)]
pub enum CollisionType {
	#[display(fmt = "opposite({}, {})", first_idx, second_idx)]
	Opposite { first_idx: usize, second_idx: usize },
	#[display(fmt = "neighbors({})", idx)]
	Neighbors { idx: usize },
	#[display(fmt = "radius0({}, {})", idx, dummy)]
	RadiusZero { idx: usize, dummy: Bend },
}

impl Segment {
	pub fn extreme(&self, next_initial: &Vec2) -> Vec2 {
		0.5 * (self.initial + *next_initial)
			+ 0.5
				* self.outward(next_initial)
				* bool_to_sign(self.bend == Bend::Outward)
	}

	pub fn outward(&self, next_initial: &Vec2) -> Vec2 {
		(*next_initial - self.initial).rotate(Vec2::NEG_Y)
	}

	pub fn ca(&self) -> Vec2 {
		self.initial - self.center
	}

	pub fn cb(&self, b_initial: &Vec2) -> Vec2 {
		*b_initial - self.center
	}

	pub fn radius(&self) -> f32 {
		self.ca().length()
	}

	pub fn angle(&self, next_initial: &Vec2) -> f32 {
		angle_gen(&self.ca(), &self.cb(next_initial), self.bend)
	}

	pub fn angle_a(&self) -> f32 {
		let ca = self.ca();
		f32::atan2(ca.y, ca.x)
	}

	pub fn angle_b(&self, next_initial: &Vec2) -> f32 {
		let cb = self.cb(next_initial);
		f32::atan2(cb.y, cb.x)
	}

	pub fn circle(&self) -> Circle {
		FloatVec2 { v: self.center, f: self.radius() }
	}

	pub fn circle_neg_r(&self) -> Circle {
		FloatVec2 {
			v: self.center,
			f: self.radius() * bool_to_sign(self.bend == Bend::Inward),
		}
	}
}

pub fn angle_gen(ca: &Vec2, cb: &Vec2, bend: Bend) -> f32 {
	if bend == Bend::Outward {
		angle_counter_clockwise(ca, cb)
	} else {
		angle_counter_clockwise(cb, ca)
	}
}

pub fn draw_segment(
	a: &Segment,
	b_initial: &Vec2,
	gizmos: &mut Gizmos,
	color: &Color,
) {
	gizmos.circle_2d(a.initial, 2.0, Color::BLACK);
	gizmos.circle_2d(*b_initial, 4.0, Color::GRAY);
	gizmos.arc_2d(
		Vec2::from_array(a.center.into()),
		a.outward(b_initial).angle_between(Vec2::Y)
			+ (a.bend == Bend::Inward).then_some(PI).unwrap_or(0.0),
		a.angle(b_initial),
		a.radius(),
		*color,
	);
}
