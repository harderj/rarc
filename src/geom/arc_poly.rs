use std::f32::consts::PI;

use bevy::{
	ecs::{component::Component, system::Resource},
	gizmos::gizmos::Gizmos,
	math::Vec2,
	reflect::Reflect,
	render::color::Color,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, UnitDisc};

use crate::geom::arc_poly::CollisionType::Opposite;
use crate::math::{Collision, FloatVec2};

use super::arc::Arc;

#[derive(Component, Reflect, Default, Clone)]
pub struct ArcPoly {
	pub original: Vec<Arc>,
	pub shrink: f32,
}

pub enum CollisionType {
	Opposite,
	Neighbors,
	Triangle,
}

impl ArcPoly {
	pub fn draw(&self, gizmos: &mut Gizmos, already_shrunken: bool) {
		for arc in &self.original {
			arc.draw(
				gizmos,
				if already_shrunken { Color::BLUE } else { Color::GREEN },
			); // TODO: is this right?
		}
		if !already_shrunken {
			let shr = self.shrunken(gizmos);
			for arc_poly in shr {
				arc_poly.draw(gizmos, true); // true important! otherwise stack overflow
			}
		}
	}

	pub fn collision_indices(&self) -> Vec<(Option<usize>, Option<usize>)> {
		let n = self.original.len();
		let mut v = Vec::default();
		for i in 0..self.original.len() {
			let prev = self.original[(n + i - 1) % n].clone();
			let next = self.original[(n + i + 1) % n].clone();
			v.push((
				prev.collision_idx(self.original[i]),
				self.original[i].collision_idx(next),
			));
		}
		v
	}

	pub fn future_collisions(&self) -> Vec<(CollisionType, Collision)> {
		let mut collisions = self.opposite_collisions();
		collisions.sort_by(|(_, c1), (_, c2)| c1.f.total_cmp(&c2.f));
		collisions
	}

	pub fn opposite_collisions(&self) -> Vec<(CollisionType, Collision)> {
		let mut vec: Vec<(CollisionType, Collision)> = vec![];
		let n = self.original.len();
		for i in 0..n {
			let a = &self.original[i];
			let ac = a.center();
			let ar = a.radius();
			for j in 2..n - 1 {
				let b = &self.original[(i + j) % n];
				let bc = b.center();
				let br = b.radius();
				let ab = bc - ac;
				let l = ab.length();
				vec.push((
					Opposite,
					FloatVec2 {
						v: ac + (0.5 * (l + ar - br) * (ab.normalize())),
						f: 0.5 * (l - ar - br),
					},
				));
			}
		}
		vec
	}

	pub fn shrunken(&self, gizmos: &mut Gizmos) -> Vec<ArcPoly> {
		let collisions = self.future_collisions();
		for (_, c) in collisions {
			gizmos.circle_2d(c.v, 2.0, Color::WHITE);
		}
		Vec::from([self.shrunken_naive()])
	}

	pub fn shrunken_naive(&self) -> ArcPoly {
		let n = self.original.len();
		let col_idxs = self.collision_indices();

		let mut naive_arcs = self.original.clone();

		for arc in naive_arcs.iter_mut() {
			arc.shrink_keeping_center(self.shrink);
		}

		let mut output_arcs = naive_arcs.clone();

		for i in 0..n {
			let i_p = (n + i - 1) % n;
			let i_n = (n + i + 1) % n;
			output_arcs[i].adjust_to_neighbors(
				col_idxs[i],
				&naive_arcs[i_p],
				&naive_arcs[i_n],
			);
		}
		ArcPoly { original: output_arcs, shrink: 0.0 }
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
				) * gen_input.r
					+ Vec2::from_array(UnitDisc.sample(&mut rng)) * gen_input.offset_noise
			};
			let absolute_bend = rng.gen_range(
				gen_input.bend_min
					..f32::max(gen_input.bend_min + 0.01, gen_input.bend_max),
			);
			res.original.push(Arc {
				a: previous,
				b: next,
				bend: if rng.gen_bool(0.5) { absolute_bend } else { -absolute_bend },
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
			bend_max: 0.5,
			bend_min: 0.4,
			shrink: 30.0,
		}
	}
}
