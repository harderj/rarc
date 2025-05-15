use std::f32::consts::PI;

use bevy::math::{Mat2, Vec2};

pub fn approximates(a: f32, b: f32, margin: f32) -> bool {
	(a - b).abs() < margin
}

pub fn angle_within(a: f32, min: f32, max: f32) -> bool {
	let min_ = between_0_2_pi(min);
	let max_ = between_0_2_pi(max);
	let a_ = between_0_2_pi(a);
	if min_ < max_ { min_ < a_ && a_ < max_ } else { a_ < max_ || a_ > min_ }
}

pub fn between_0_2_pi(a: f32) -> f32 {
	(2.0 * PI + (2.0 * PI + a) % (2.0 * PI)) % (2.0 * PI)
}

pub fn between_clockwise(angle: f32, from: f32, to: f32) -> bool {
	let angle_ = between_0_2_pi(angle);
	let from_ = between_0_2_pi(from);
	let to_ = between_0_2_pi(to);
	if from_ < to_ {
		angle_ <= from_ || angle_ > to_
	} else {
		angle_ <= from_ && angle_ > to_
	}
}

pub fn between_counterclockwise(angle: f32, from: f32, to: f32) -> bool {
	!between_clockwise(angle, from, to)
}

pub fn clockwise_difference(a: f32, b: f32) -> f32 {
	between_0_2_pi(between_0_2_pi(a) - between_0_2_pi(b))
}

pub fn counterclockwise_difference(a: f32, b: f32) -> f32 {
	between_0_2_pi(between_0_2_pi(b) - between_0_2_pi(a))
}

pub fn midpoint(a: Vec2, b: Vec2) -> Vec2 {
	0.5 * (a + b)
}

pub fn bend_to_abs_angle(bend: f32) -> f32 {
	2.0 * f32::acos(1.0 - bend.abs())
}

pub fn second_deg_eq(a: f32, b: f32, c: f32) -> Vec<f32> {
	let d = b.powi(2) - 4.0 * a * c;
	if d < 0.0 {
		Vec::new()
	} else if d == 0.0 {
		Vec::from([-b / (2.0 * a)])
	} else {
		let sqrt_d = d.sqrt();
		let v: Vec2 = (Vec2::new(-sqrt_d, sqrt_d) - b) / (2.0 * a);
		Vec::from([v.min_element(), v.max_element()])
	}
}

pub fn angle_counter_clockwise(a: Vec2, b: Vec2) -> f32 {
	(Mat2::from_cols(a, b).determinant().atan2(a.dot(b)) + 2.0 * PI) % (2.0 * PI)
}

pub fn bool_to_sign(b: bool) -> f32 {
	if b { 1.0 } else { -1.0 }
}
