use std::{
	f32::consts::PI,
	fmt::{Display, Formatter, Result},
};

extern crate derive_more;
use derive_more::Display;

use bevy::{
	ecs::{component::Component, system::Resource},
	gizmos::gizmos::Gizmos,
	math::Vec2,
	prelude::default,
	reflect::Reflect,
	render::color::Color,
	utils::petgraph::adj::Neighbors,
};
use itertools::Itertools;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, UnitDisc};

use super::arc::Arc;
use crate::math::{three_circle_collision, FloatVec2};
use crate::{
	geom::arc_poly::CollisionType::Opposite, math::angle_counter_clockwise,
};

#[derive(Component, Reflect, Default, Clone)]
pub struct ArcPoly {
	pub original: Vec<Arc>,
	pub shrink: f32,
}

impl Display for ArcPoly {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "arc_poly(shrink={}, links=[\n", self.shrink)?;
		for arc in self.original.iter() {
			write!(f, "	{},\n", arc)?;
		}
		write!(f, "])")
	}
}

#[derive(Display)]
#[display(fmt = "collision({}, {})", kind, time_place)]
pub struct Collision {
	time_place: FloatVec2,
	kind: CollisionType,
}

pub enum CollisionType {
	Opposite { first_idx: usize, second_idx: usize },
	Neighbors { idx: usize },
}

impl Display for CollisionType {
	fn fmt(&self, f: &mut Formatter) -> Result {
		match self {
			CollisionType::Opposite { first_idx, second_idx } => {
				write!(f, "opposite({}, {})", first_idx, second_idx)
			}
			CollisionType::Neighbors { idx } => write!(f, "neighbors({})", idx),
		}
	}
}

impl ArcPoly {
	pub fn draw(&self, gizmos: &mut Gizmos, already_shrunk: bool) {
		for arc in &self.original {
			arc.draw(gizmos, if already_shrunk { Color::BLUE } else { Color::GREEN }); // TODO: is this right?
		}
		if !already_shrunk {
			let shr = self.shrunk(gizmos);
			for arc_poly in shr {
				arc_poly.draw(gizmos, true); // true important! otherwise stack overflow
			}
		}
		// panic!()
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
		collisions.append(&mut self.neighbor_collisions());
		collisions.sort_by(|c1, c2| c1.time_place.f.total_cmp(&c2.time_place.f));
		collisions
	}

	pub fn neighbor_collisions(&self) -> Vec<Collision> {
		let mut vec: Vec<Collision> = vec![];
		let n = self.original.len();
		for i in 0..n {
			let prev = &self.original[(n - 1 + i) % n];
			let this = &self.original[(n + 0 + i) % n];
			let next = &self.original[(n + 1 + i) % n];
			let pcol = three_circle_collision(
				&prev.circle_neg_r(),
				&this.circle_neg_r(),
				&next.circle_neg_r(),
			);
			if let Some(c) = pcol {
				if c.f > 0.0 {
					vec.push(Collision {
						time_place: c,
						kind: CollisionType::Neighbors { idx: i },
					});
				}
			}
		}
		vec
	}

	pub fn opposite_collisions(&self) -> Vec<Collision> {
		let mut vec: Vec<Collision> = vec![];
		let n = self.original.len();
		if n <= 3 {
			return vec![];
		}
		for i in 0..n {
			let first = &self.original[i];
			let first_c = first.center();
			let first_r = first.radius();
			for j in i + 2..n {
				if i == 0 && j == n - 1 {
					continue;
				}
				let second = &self.original[j];
				if first.bend < 0.0 && second.bend < 0.0 {
					let second_c = second.center();
					let second_r = second.radius();
					let center_line = second_c - first_c;
					let dist = center_line.length();
					let t = 0.5 * (dist - first_r - second_r);
					if t >= 0.0 && t < self.shrink {
						let place = first_c + (first_r + t) * center_line.normalize();
						let naive = self.shrink_naive(t);
						let first_naive = naive.original[i];
						let second_naive = naive.original[j];
						let first_naive_c = first_naive.center();
						let second_naive_c = second_naive.center();
						let [fbv, fba, sbv, sba] = [
							angle_counter_clockwise(
								first_naive.b - first_naive_c,
								place - first_naive_c,
							),
							angle_counter_clockwise(
								first_naive.b - first_naive_c,
								first_naive.a - first_naive_c,
							),
							angle_counter_clockwise(
								second_naive.b - second_naive_c,
								place - second_naive_c,
							),
							angle_counter_clockwise(
								second_naive.b - second_naive_c,
								second_naive.a - second_naive_c,
							),
						];
						if fbv < fba && sbv < sba {
							let col = Collision {
								time_place: FloatVec2 { f: t, v: place },
								kind: Opposite { first_idx: i, second_idx: j },
							};

							vec.push(col);
						}
					}
				}
				// TODO: else..
			}
		}
		vec
	}

	pub fn max_arc_length(&self) -> f32 {
		self
			.original
			.iter()
			.map(|a| a.ab().length())
			.reduce(f32::max)
			.unwrap_or(f32::MAX)
	}

	pub fn shrunk(&self, gizmos: &mut Gizmos) -> Vec<ArcPoly> {
		let collisions = self.future_collisions();
		if let Some(c) = collisions.first() {
			if 0.0 < c.time_place.f && c.time_place.f <= self.shrink {
				let shrunk = self.shrink_naive(c.time_place.f);
				if self.original.len() <= 3 {
					return vec![];
				}
				let children = match c.kind {
					Opposite { first_idx: first, second_idx: second } => {
						println!("opposite");
						split_opposite(shrunk, c.time_place.v, first, second)
					}
					CollisionType::Neighbors { idx: i } => {
						println!("neighbor");
						vec![shrunk.with_removed(i)]
					}
				};
				return children.iter().flat_map(|x| x.shrunk(gizmos)).collect_vec();
			}
		}
		vec![self.shrink_naive(self.shrink)]
	}

	pub fn with_removed(&self, idx: usize) -> ArcPoly {
		let mut clone = self.clone();
		clone.original.remove(idx);
		clone
	}

	pub fn shrink_naive(&self, shrink: f32) -> ArcPoly {
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
		ArcPoly { original: output_arcs, shrink: self.shrink - shrink }
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
				bend: -absolute_bend, // if rng.gen_bool(0.5) { absolute_bend } else { -absolute_bend },
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
) -> Vec<ArcPoly> {
	let n = arc_poly.original.len();
	let mut j: usize = 0;
	let mut polys: Vec<ArcPoly> = vec![default(), default()];
	polys.iter_mut().for_each(|p| p.shrink = arc_poly.shrink);
	for i in 0..n {
		let arc = arc_poly.original[i].clone();
		if [first_idx, second_idx].contains(&i) {
			let mut arc_left = arc.clone();
			let mut arc_right = arc.clone();
			arc_left.set_b_keeping_center(place);
			arc_right.set_a_keeping_center(place);
			polys[j].original.push(arc_left);
			j = (j + 1) % 2;
			polys[j].original.push(arc_right);
		} else {
			polys[j].original.push(arc);
		}
	}
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
			random_seed: 0,
			n: 3,
			r: 340.5,
			offset_noise: 74.1,
			bend_max: 0.5,
			bend_min: 0.02,
			shrink: 101.8,
		}
	}
}
