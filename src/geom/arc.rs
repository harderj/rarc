use std::f32::consts::PI;

use crate::math::{
	circle_center_from_3_points, two_circle_collision, Circle, FloatVec2
};
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
		let mut r = Mat2::from_cols(self.ca(), self.cb())
			.determinant()
			.atan2(self.ca().dot(self.cb()))
			* f32::signum(self.bend);
		if r < 0.0 {
			r += 2.0 * PI
		}
		r
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
		FloatVec2 {
			v: self.center(),
			f: self.radius(),
		}
	}

	fn collision_idx(&self, other: Arc) -> Option<usize> {
		const TOLERANCE: f32 = 0.001;
		let cols = two_circle_collision(self.circle(), other.circle());
		if cols.len() < 2 { None } else {
			let b_dist_0 = (cols[0] - self.b).length();
			let b_dist_1 = (cols[1] - self.b).length();
			if b_dist_0 < TOLERANCE { Some(0) }
			else if b_dist_1 < TOLERANCE { Some(1) }
			else { None }
		}
	}

	fn adjust_b(&mut self, next: Arc, col_idx: usize) -> Option<Vec2> {
		let cols_next = two_circle_collision(self.circle(), next.circle());
		if cols_next.len() > 1 {
			let col = cols_next[col_idx];
			self.b = col;
			Some(col)
		} else {
			None
		}
	}

	fn shrink(&mut self, amount: f32) {
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
		gizmos.circle_2d(
			Vec2::from_array(self.b.into()),
			4.0,
			Color::GRAY,
		);
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

#[derive(Component, Reflect, Default, Clone)]
pub struct ArcPoly {
	pub original: Vec<Arc>,
	pub shrink: f32
}

impl ArcPoly {
	pub fn draw(&self, gizmos: &mut Gizmos, already_shrunk: bool) {
		for arc in self.original.iter() {
			arc.downcast_ref::<Arc>().unwrap().draw(
				gizmos,
				if already_shrunk {
					Color::BLUE
				} else {
					Color::GREEN
				},
			); // TODO: is this right?
		}
		if !already_shrunk {
			let shr = self.shrunk();
			for arc_poly in shr {
				arc_poly.draw(gizmos, true); // true important! otherwise stack overflow
			}
		}
	}

	pub fn collision_indices(&self) -> Vec<Option<usize>> {
		let n = self.original.len();
		let mut v = Vec::default();
		for i in 0..self.original.len() {
			let next = self.original[(i + 1) % n].clone();
			v.push(self.original[i].collision_idx(next));
		}
		v
	}

	pub fn shrunk(&self) -> Vec<ArcPoly> {
		let n = self.original.len();
		let col_idxs = self.collision_indices();

		let mut naive_arcs = self.original.clone();

		for arc in naive_arcs.iter_mut() {
			arc.shrink(self.shrink);
		}

		let mut output_arcs = naive_arcs.clone();

		for i in 0..n {
			if col_idxs[i].is_some() {
				// println!("{}", i);
				let j = (i + 1) % n;
				let col = output_arcs[i].adjust_b(
					naive_arcs[j].clone(), col_idxs[i].unwrap()
				);
				if col.is_some() { output_arcs[j].a = col.unwrap() };
			}
		}
		Vec::from([ArcPoly {
			original: output_arcs,
			shrink: 0.0,
		}])
	}

	pub fn from_gen_input(gen_input: &ArcPolyGenInput) -> Self {
		let mut rng = StdRng::seed_from_u64(gen_input.random_seed as u64);
		let mut previous = Vec2::X * gen_input.r;
		let mut res = ArcPoly::default();
		res.shrink = gen_input.shrink;
		for i in 1..(gen_input.n + 1) {
			let next = if i == gen_input.n {
				res.original[0].a
			} else {
				Vec2::new(
					f32::cos(2.0 * PI * (i as f32) / (gen_input.n as f32)),
					f32::sin(2.0 * PI * (i as f32) / (gen_input.n as f32)),
				) * gen_input.r + Vec2::from_array(UnitDisc.sample(&mut rng))
					* gen_input.offset_noise
			};
			let absolute_bend = rng.gen_range(
				gen_input.bend_min
					..f32::max(gen_input.bend_min + 0.01, gen_input.bend_max),
			);
			res.original.push(Arc {
				a: previous,
				b: next,
				bend: if rng.gen_bool(0.5) {
					absolute_bend
				} else {
					-absolute_bend
				},
			});
			previous = next;
		}
		res
	}
}

#[derive(Reflect, Resource)]
pub struct ArcPolyGenInput {
	random_seed: u32,
	n: i32,
	r: f32,
	offset_noise: f32,
	bend_max: f32,
	bend_min: f32,
	shrink: f32,
}

impl Default for ArcPolyGenInput {
	fn default() -> Self {
		ArcPolyGenInput {
			random_seed: 0,
			n: 5,
			r: 200.0,
			offset_noise: 30.0,
			bend_max: 0.3,
			bend_min: 0.2,
			shrink: 30.0,
		}
	}
}
