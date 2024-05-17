extern crate derive_more;
use derive_more::{Add, Display, Sub};

use std::f32::consts::PI;

use bevy::{
	ecs::component::Component,
	math::{Mat2, Mat3, Vec2, Vec3},
	reflect::Reflect,
};

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

pub fn circle_center_from_3_points(p1: &Vec2, p2: &Vec2, p3: &Vec2) -> Vec2 {
	let c1 =
		Vec3::new(p1.length_squared(), p2.length_squared(), p3.length_squared());
	let c2 = Vec3::new(p1.x, p2.x, p3.x);
	let c3 = Vec3::new(p1.y, p2.y, p3.y);

	let m1 = Mat3::from_cols(c2, c3, Vec3::ONE);
	let m2 = Mat3::from_cols(c1, c3, Vec3::ONE);
	let m3 = Mat3::from_cols(c1, c2, Vec3::ONE);

	Vec2::new(m2.determinant(), -m3.determinant()) * 0.5 / m1.determinant()
}

#[derive(Clone, Component, Copy, Display, Add, Reflect, Sub)]
#[display(fmt = "({}, {})", f, v)]
pub struct FloatVec2 {
	pub f: f32,
	pub v: Vec2,
}

pub type Circle = FloatVec2;

pub fn angle_counter_clockwise(a: &Vec2, b: &Vec2) -> f32 {
	(Mat2::from_cols(*a, *b).determinant().atan2(a.dot(*b)) + 2.0 * PI)
		% (2.0 * PI)
}

pub fn bool_to_sign(b: bool) -> f32 {
	if b {
		1.0
	} else {
		-1.0
	}
}

pub fn two_circle_collision(a: &Circle, b: &Circle) -> Vec<Vec2> {
	let d = (a.v - b.v).length();
	if d > a.f + b.f || d < f32::abs(a.f - b.f) || d == 0.0 {
		Vec::default()
	} else if d == a.f + b.f {
		Vec::from([a.v + (b.v - a.v).normalize() * a.f])
	} else {
		let alpha = (a.f.powi(2) - b.f.powi(2) + d.powi(2)) / (2.0 * d);
		let h = (a.f.powi(2) - alpha.powi(2)).sqrt();
		let v2 = a.v + alpha * (b.v - a.v) / d;
		let mut v3 = Mat2::from_cols(Vec2::Y, Vec2::X) * (h * (b.v - a.v) / d);
		v3.y *= -1.0;
		Vec::from([v2 + v3, v2 - v3])
	}
}

pub fn three_circle_collision(
	a: &Circle,
	b: &Circle,
	c: &Circle,
) -> Option<FloatVec2> {
	let a_ = *a - *c;
	let b_ = *b - *c;
	let pcol = three_circle_collision_0(&a_, &b_);
	match pcol {
		None => None,
		Some(col) => Some(FloatVec2 { f: col.f - c.f, v: col.v + c.v }),
	}
}

fn three_circle_collision_0(a: &Circle, b: &Circle) -> Option<FloatVec2> {
	let m = Mat2::from_cols(a.v, b.v).transpose();
	let alpha = 1.0 / (2.0 * m.determinant());
	let beta_a = a.v.length_squared() - a.f.powi(2);
	let beta_b = b.v.length_squared() - b.f.powi(2);
	let gamma_a = -2.0 * a.f;
	let gamma_b = -2.0 * b.f;
	let delta_x = alpha * (b.v.y * gamma_a - a.v.y * gamma_b);
	let delta_y = alpha * (-b.v.x * gamma_a + a.v.x * gamma_b);
	let epsilon_x = alpha * (b.v.y * beta_a - a.v.y * beta_b);
	let epsilon_y = alpha * (-b.v.x * beta_a + a.v.x * beta_b);
	let eq_a = delta_x.powi(2) + delta_y.powi(2) - 1.0;
	let eq_b = 2.0 * (delta_x * epsilon_x + delta_y * epsilon_y);
	let eq_c = epsilon_x.powi(2) + epsilon_y.powi(2);
	let pot_ts = second_deg_eq(eq_a, eq_b, eq_c);
	let pot_t = match pot_ts.len() {
		0 => None,
		1 => {
			let t = *pot_ts.first().unwrap();
			if t > 0.0 {
				Some(t)
			} else {
				None
			}
		}
		2 => {
			let mut t: f32 = *pot_ts.first().unwrap();
			if t < 0.0 {
				t = *pot_ts.get(1).unwrap();
			}
			if t > 0.0 {
				Some(t)
			} else {
				None
			}
		}
		_ => panic!("Not possible."),
	};
	match pot_t {
		None => None,
		Some(t) => Some(FloatVec2 {
			f: t,
			v: Vec2::new(delta_x * t + epsilon_x, delta_y * t + epsilon_y),
		}),
	}
}
