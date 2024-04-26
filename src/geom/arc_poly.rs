use std::{borrow::Borrow, f32::consts::PI};

use bevy::{
	ecs::{component::Component, system::Resource},
	gizmos::gizmos::Gizmos,
	math::Vec2,
	prelude::default,
	reflect::Reflect,
	render::color::Color,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, UnitDisc};

use super::arc::Arc;
use crate::geom::arc_poly::CollisionType::Opposite;
use crate::math::FloatVec2;

#[derive(Component, Reflect, Default, Clone)]
pub struct ArcPoly {
	pub original: Vec<Arc>,
	pub shrink: f32,
}

pub struct Collision {
	time_place: FloatVec2,
	kind: CollisionType,
}

pub enum CollisionType {
	Opposite { first_idx: usize, second_idx: usize },
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
			let shr = self.shrunk(gizmos);
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

	pub fn future_collisions(&self) -> Vec<Collision> {
		let mut collisions: Vec<Collision> = self.opposite_collisions();
		collisions.sort_by(|c1, c2| c1.time_place.f.total_cmp(&c2.time_place.f));
		collisions
	}

	pub fn opposite_collisions(&self) -> Vec<Collision> {
		let mut vec: Vec<Collision> = vec![];
		let n = self.original.len();
		for i in 0..n {
			let first = &self.original[i];
			let first_c = first.center();
			let first_r = first.radius();
			for j in 2..n - 2 {
				let second = &self.original[(i + j) % n];
				// println!("first.bend: {}, second.bend: {}", first.bend, second.bend);
				if first.bend < 0.0 && second.bend < 0.0 {
					let second_c = second.center();
					let second_r = second.radius();
					let center_line = second_c - first_c;
					let dist = center_line.length();
					let t = 0.5 * (dist - first_r - second_r);
					if t >= 0.0 {
						let fv = FloatVec2 {
							v: first_c + (first_r + t) * center_line.normalize(),
							f: t,
						};
						vec.push(Collision {
							time_place: fv,
							kind: Opposite { first_idx: i, second_idx: j },
						});
					}
				}
				// TODO: else?
			}
		}
		vec
	}

	pub fn shrunk(&self, gizmos: &mut Gizmos) -> Vec<ArcPoly> {
		let collisions = self.future_collisions();
		for c in collisions.iter() {
			gizmos.circle_2d(c.time_place.v, 2.0, Color::WHITE);
		}
		collisions
			.first()
			.map(|c| {
				if c.time_place.f <= self.shrink {
					let aps = match c.kind {
						Opposite { first_idx: first, second_idx: second } => {
							split_opposite(
								self.shrunken_naive(c.time_place.f),
								c.time_place.v,
								first,
								second,
								self.shrink - c.time_place.f,
							)
						}
						_ => todo!(),
					};
					Some(aps.map(|x| x.shrunk(gizmos)).concat())
				} else {
					None
				}
			})
			.flatten()
			.unwrap_or(vec![self.shrunken_naive(self.shrink)])
	}

	pub fn shrunken_naive(&self, shrink: f32) -> ArcPoly {
		let n = self.original.len();
		let col_idxs = self.collision_indices();

		let mut naive_arcs = self.original.clone();

		for arc in naive_arcs.iter_mut() {
			arc.shrink_keeping_center(shrink);
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

pub fn split_opposite(
	arc_poly: ArcPoly,
	place: Vec2,
	first_idx: usize,
	second_idx: usize,
	shrink: f32,
) -> [ArcPoly; 2] {
	let n = arc_poly.original.len();
	let mut j: usize = 0;
	let mut polys: [ArcPoly; 2] = default();
	for i in 0..n {
		let arc = arc_poly.original[i].borrow();
		if [first_idx, second_idx].contains(&i) {
			let mut arc_left = arc.clone();
			let mut arc_right = arc.clone();
			arc_left.set_b_keeping_center(place);
			arc_right.set_a_keeping_center(place);
			polys[j].original.push(arc_left);
			j = (j + 1) % 2;
			polys[j].original.push(arc_right);
		} else {
			polys[j].original.push(*arc);
		}
	}
	polys.iter_mut().for_each(|p| p.shrink = shrink);
	polys
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
			random_seed: 15,
			n: 5,
			r: 200.0,
			offset_noise: 30.0,
			bend_max: 0.8,
			bend_min: 0.4,
			shrink: 30.0,
		}
	}
}
