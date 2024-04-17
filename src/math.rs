use derive_more::{Add, Sub};
use itertools::Itertools;

use std::f32::consts::PI;

use bevy::{
	ecs::component::Component,
	math::{Mat2, Mat3, Vec2, Vec3},
	reflect::Reflect,
};

#[derive(Clone, Component, Copy, Add, Debug, Reflect, Sub, Default)]
pub struct FloatVec2(pub f32, pub Vec2);

pub type Circle = FloatVec2;

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
	(2.0 * PI + a) % (2.0 * PI)
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

pub fn circle_center_from_3_points(p1: Vec2, p2: Vec2, p3: Vec2) -> Vec2 {
	let c1 =
		Vec3::new(p1.length_squared(), p2.length_squared(), p3.length_squared());
	let c2 = Vec3::new(p1.x, p2.x, p3.x);
	let c3 = Vec3::new(p1.y, p2.y, p3.y);

	let m1 = Mat3::from_cols(c2, c3, Vec3::ONE);
	let m2 = Mat3::from_cols(c1, c3, Vec3::ONE);
	let m3 = Mat3::from_cols(c1, c2, Vec3::ONE);

	Vec2::new(m2.determinant(), -m3.determinant()) * 0.5 / m1.determinant()
}

pub fn circle_from_endpoints_and_bend(a: Vec2, b: Vec2, bend: f32) -> Circle {
	let crd = (b - a).length(); // chord
	let perp = ((b - a) / crd).rotate(Vec2::Y);
	let mid = midpoint(a, b);
	let radius = crd / (2.0 * f32::sqrt((2.0 - bend) * bend));
	let arc_mid = mid + perp * bend * radius;
	let center = circle_center_from_3_points(a, b, arc_mid);
	FloatVec2(radius, center)
}

pub fn angle_counter_clockwise(a: &Vec2, b: &Vec2) -> f32 {
	(Mat2::from_cols(*a, *b).determinant().atan2(a.dot(*b)) + 2.0 * PI)
		% (2.0 * PI)
}

pub fn bool_to_sign(b: bool) -> f32 {
	if b { 1.0 } else { -1.0 }
}

pub fn two_circle_collision(a: Circle, b: Circle) -> Vec<Vec2> {
	let FloatVec2(r_a, c_a) = a;
	let FloatVec2(r_b, c_b) = b;
	let d = (c_a - c_b).length();
	if d > r_a + r_b || d < f32::abs(r_a - r_b) || d == 0.0 {
		Vec::default()
	} else if d == r_a + r_b {
		Vec::from([c_a + (c_b - c_a).normalize() * r_a])
	} else {
		let alpha = (r_a.powi(2) - r_b.powi(2) + d.powi(2)) / (2.0 * d);
		let h = (r_a.powi(2) - alpha.powi(2)).sqrt();
		let v2 = c_a + alpha * (c_b - c_a) / d;
		let mut v3 = Mat2::from_cols(Vec2::Y, Vec2::X) * (h * (c_b - c_a) / d);
		v3.y *= -1.0;
		Vec::from([v2 + v3, v2 - v3])
	}
}

pub fn three_circle_collision(
	a: Circle,
	b: Circle,
	c: Circle,
) -> Vec<FloatVec2> {
	let a_ = a - c;
	let b_ = b - c;
	let FloatVec2(cf, cv) = c;
	three_circle_collision_0(&a_, &b_)
		.iter()
		.map(|FloatVec2(f, v)| FloatVec2(f - cf, *v + cv))
		.collect_vec()
}

fn three_circle_collision_0(a: &Circle, b: &Circle) -> Vec<FloatVec2> {
	let FloatVec2(r_a, c_a) = a;
	let FloatVec2(r_b, c_b) = b;
	let m = Mat2::from_cols(*c_a, *c_b).transpose();
	let alpha = 1.0 / (2.0 * m.determinant());
	let beta_a = c_a.length_squared() - r_a.powi(2);
	let beta_b = c_b.length_squared() - r_b.powi(2);
	let gamma_a = -2.0 * r_a;
	let gamma_b = -2.0 * r_b;
	let delta_x = alpha * (c_b.y * gamma_a - c_a.y * gamma_b);
	let delta_y = alpha * (-c_b.x * gamma_a + c_a.x * gamma_b);
	let epsilon_x = alpha * (c_b.y * beta_a - c_a.y * beta_b);
	let epsilon_y = alpha * (-c_b.x * beta_a + c_a.x * beta_b);
	let eq_a = delta_x.powi(2) + delta_y.powi(2) - 1.0;
	let eq_b = 2.0 * (delta_x * epsilon_x + delta_y * epsilon_y);
	let eq_c = epsilon_x.powi(2) + epsilon_y.powi(2);
	second_deg_eq(eq_a, eq_b, eq_c)
		.iter()
		.map(|t| {
			FloatVec2(*t, Vec2::new(delta_x * t + epsilon_x, delta_y * t + epsilon_y))
		})
		.collect_vec()
}
