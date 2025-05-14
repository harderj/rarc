use bevy::{
	color::Color,
	ecs::component::Component,
	gizmos::gizmos::Gizmos,
	math::{Isometry2d, Mat2, Mat3, Vec2, Vec3},
	reflect::Reflect,
};
use derive_more::{Add, Sub};

use crate::{
	geom::misc::DrawableWithGizmos,
	math::{midpoint, second_deg_eq},
};

const CIRCLE_RESOLUTION: u32 = 128;

#[derive(Clone, Component, Copy, Add, Debug, Reflect, Sub, Default)]
pub struct Circle {
	pub radius: f32,
	pub center: Vec2,
}

impl DrawableWithGizmos for Circle {
	fn draw_gizmos(&self, gizmos: &mut Gizmos, color: Color) {
		gizmos
			.circle_2d(
				Isometry2d { rotation: Default::default(), translation: self.center },
				self.radius,
				color,
			)
			.resolution(CIRCLE_RESOLUTION);
	}
}

impl Circle {
	pub fn new(radius: f32, center: Vec2) -> Self {
		Self { radius, center }
	}

	pub fn from_3_points(p1: Vec2, p2: Vec2, p3: Vec2) -> Self {
		let c1 =
			Vec3::new(p1.length_squared(), p2.length_squared(), p3.length_squared());
		let c2 = Vec3::new(p1.x, p2.x, p3.x);
		let c3 = Vec3::new(p1.y, p2.y, p3.y);

		let m1 = Mat3::from_cols(c2, c3, Vec3::ONE);
		let m2 = Mat3::from_cols(c1, c3, Vec3::ONE);
		let m3 = Mat3::from_cols(c1, c2, Vec3::ONE);

		let center =
			Vec2::new(m2.determinant(), -m3.determinant()) * 0.5 / m1.determinant();
		let radius = center.distance(p1);
		Self { radius, center }
	}

	pub fn from_endpoints_and_bend(a: Vec2, b: Vec2, bend: f32) -> Circle {
		let crd = (b - a).length(); // chord
		let perp = ((b - a) / crd).rotate(Vec2::Y);
		let mid = midpoint(a, b);
		let radius = crd / (2.0 * f32::sqrt((2.0 - bend) * bend));
		let arc_mid = mid + perp * bend * radius;
		Self::from_3_points(a, b, arc_mid)
	}

	pub fn intersect(self, other: Circle) -> Vec<Vec2> {
		let (a, b) = (self, other);
		let Circle { radius: r_a, center: c_a } = a;
		let Circle { radius: r_b, center: c_b } = b;
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

	pub fn three_circle_tangent(a: Circle, b: Circle, c: Circle) -> Vec<Circle> {
		let a_ = a - c;
		let b_ = b - c;
		let Circle { radius: cf, center: cv } = c;
		a_.three_circle_tangent_0(b_)
			.iter()
			.map(|Circle { radius, center }| Circle {
				radius: radius - cf,
				center: center + cv,
			})
			.collect()
	}

	fn three_circle_tangent_0(self, other: Circle) -> Vec<Circle> {
		let (a, b) = (self, other);
		let Circle { radius: r_a, center: c_a } = a;
		let Circle { radius: r_b, center: c_b } = b;
		let m = Mat2::from_cols(c_a, c_b).transpose();
		let determinant = m.determinant();
		if determinant == 0.0 {
			return vec![];
		};
		let alpha = 1.0 / (2.0 * determinant);
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
			.into_iter()
			.map(|radius| Circle {
				radius,
				center: Vec2::new(
					delta_x * radius + epsilon_x,
					delta_y * radius + epsilon_y,
				),
			})
			.collect()
	}
}
