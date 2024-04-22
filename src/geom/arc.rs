use std::{borrow::Borrow, f32::consts::PI};

use crate::math::circle_center_from_3_points;
use bevy::{
	ecs::{component::Component, system::Resource},
	gizmos::gizmos::Gizmos,
	math::{Mat2, Vec2},
	reflect::{List, Reflect},
	render::color::Color,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, UnitDisc};

#[derive(Component, Reflect, Clone)]
pub struct Arc {
	pub a: Vec2,
	pub b: Vec2,
	pub s: f32, // arc height (positive is right of A->B)
}

impl Arc {
	pub fn ab(&self) -> Vec2 {
		self.b - self.a
	}

	pub fn sv(&self) -> Vec2 {
		self.ab().rotate(Vec2::NEG_Y).normalize() * self.s
	}

	pub fn extreme(&self) -> Vec2 {
		self.sv() + 0.5 * (self.a + self.b)
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
		let mut r = Mat2::from_cols(self.ca(), self.cb())
			.determinant()
			.atan2(self.ca().dot(self.cb()))
			* f32::signum(self.s);
		if r < 0.0 {
			r += 2.0 * PI
		}
		r
	}

	pub fn angle_a(&self) -> f32 {
		let ca = self.ca();
		f32::atan2(ca.y, ca.x)
		//self.ca().angle_between(Vec2::X)
	}

	pub fn angle_b(&self) -> f32 {
		let cb = self.cb();
		f32::atan2(cb.y, cb.x)
		// self.cb().angle_between(Vec2::X)
	}

	fn shrink(&mut self, amount: f32, neighbors: (&Arc, &Arc)) {
		let r = self.radius();
		let c = self.center();
		let ang_a = self.angle_a();
		let ang_b = self.angle_b();
		let new_a =
			c + (r - amount) * Vec2::new(f32::cos(ang_a), f32::sin(ang_a));
		let new_b =
			c + (r - amount) * Vec2::new(f32::cos(ang_b), f32::sin(ang_b));
		self.a = new_a;
		self.b = new_b;
	}

	pub fn draw(&self, gizmos: &mut Gizmos, color: Color) {
		gizmos.circle_2d(Vec2::from_array(self.a.into()), 2.0, Color::GRAY);
		gizmos.circle_2d(
			Vec2::from_array(self.b.into()),
			4.0,
			Color::DARK_GRAY,
		);
		gizmos.arc_2d(
			Vec2::from_array(self.center().into()),
			self.sv().angle_between(Vec2::Y),
			self.angle(),
			self.radius(),
			color,
		);
	}
}

#[derive(Component, Reflect, Default, Clone)]
pub struct ArcPoly {
	pub original: Vec<Arc>,
	pub shrink: f32,
}

impl ArcPoly {
	pub fn draw(&self, gizmos: &mut Gizmos, already_shrunk: bool) {
		for arc in self.original.iter() {
			arc.downcast_ref::<Arc>().unwrap().draw(gizmos, if already_shrunk { Color::BLUE } else { Color::GREEN }); // TODO: is this right?
		}
		if !already_shrunk {
			let shr = self.shrunk();
			for arc_poly in shr {
				arc_poly.draw(gizmos, true); // true important! otherwise stack overflow
			}
		}
	}

	pub fn shrunk(&self) -> Vec<ArcPoly> {
		let mut arcs = self.original.clone();
		let n = arcs.len();
		let i: usize = 0;
		for arc in arcs.iter_mut() {
			arc.shrink(
				self.shrink,
				(
					self.original[n + i - 1 % n].borrow(),
					self.original[i + 1 % n].borrow(),
				),
			);
		}
		Vec::from([ArcPoly {
			original: arcs,
			shrink: 0.0,
		}])
	}

	pub fn from_gen_input(gen_input: &ArcPolyGenInput) -> Self {
		let mut rng = StdRng::seed_from_u64(gen_input.random_seed as u64);
		let mut previous = Vec2::X * gen_input.r;
		let mut res = ArcPoly::default();
		res.shrink = gen_input.shrink;
		for i in 1..gen_input.n {
			let next = Vec2::new(
				f32::cos(2.0 * PI * (i as f32) / (gen_input.n as f32)),
				f32::sin(2.0 * PI * (i as f32) / (gen_input.n as f32)),
			) * gen_input.r + Vec2::from_array(
				UnitDisc.sample(&mut rng),
			) * gen_input.offset_noise;
			res.original.push(Arc {
				a: previous,
				b: next,
				s: rng.gen::<f32>() * gen_input.s_noise,
			});
			previous = next;
		}
		res.original.push(Arc {
			a: previous,
			b: Vec2::X * gen_input.r,
			s: rng.gen::<f32>() * gen_input.s_noise,
		});
		res
	}
}

#[derive(Reflect, Resource)]
pub struct ArcPolyGenInput {
	random_seed: u32,
	n: i32,
	r: f32,
	offset_noise: f32,
	s_noise: f32,
	shrink: f32,
}

impl Default for ArcPolyGenInput {
	fn default() -> Self {
		ArcPolyGenInput {
			random_seed: 0,
			n: 5,
			r: 200.0,
			offset_noise: 30.0,
			s_noise: 30.0,
			shrink: 0.0,
		}
	}
}
