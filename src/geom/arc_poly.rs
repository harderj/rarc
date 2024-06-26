use std::{
	f32::consts::PI,
	fmt::{Display, Formatter, Result},
};

use bevy::{
	ecs::{component::Component, system::Resource},
	gizmos::gizmos::Gizmos,
	math::Vec2,
	prelude::default,
	reflect::Reflect,
	render::color::Color,
};
use itertools::Itertools;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, UnitDisc};

use crate::{
	geom::segment::CollisionType,
	math::{
		angle_counter_clockwise, bool_to_sign, circle_center_from_3_points,
		midpoint, three_circle_collision, two_circle_collision, FloatVec2,
	},
};

use super::segment::{draw_segment, Bend, Collision, Segment};

#[derive(Component, Reflect, Default, Clone)]
pub struct ArcPoly {
	pub segments: Vec<Segment>,
}

impl Display for ArcPoly {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "arc_poly([\n")?;
		for arc in self.segments.iter() {
			write!(f, "	{},\n", arc)?;
		}
		write!(f, "])")
	}
}

impl ArcPoly {
	pub fn draw(&self, gizmos: &mut Gizmos, color: &Color) {
		for (i, j) in (0..self.segments.len()).circular_tuple_windows() {
			let (a, b) = (&self.segments[i], &self.segments[j]);
			draw_segment(a, &b.initial, gizmos, color);
		}
	}

	pub fn shrunk(&self, gizmos: &mut Gizmos, amount: f32) -> Vec<ArcPoly> {
		let collisions = self.future_collisions();
		if let Some(c) = collisions.first() {
			let t = c.time_place.f;
			if 0.0 < t && t < amount {
				let shrunk = self.shrink_naive(t + f32::EPSILON);
				let n = self.segments.len();
				if n <= 3 {
					return vec![];
				}
				let children = match c.kind {
					CollisionType::Opposite { first_idx: first, second_idx: second } => {
						split_opposite(shrunk, c.time_place.v, first, second)
					}
					CollisionType::Neighbors { idx: i } => {
						vec![shrunk.with_removed(i)]
					}
				};
				return children
					.iter()
					.flat_map(|x| x.shrunk(gizmos, amount - t))
					.collect_vec();
			}
		}
		vec![self.shrink_naive(amount)]
	}

	pub fn future_collisions(&self) -> Vec<Collision> {
		let mut collisions: Vec<Collision> = self.opposite_collisions();
		collisions.append(&mut self.neighbor_collisions());
		collisions.sort_by(|c1, c2| c1.time_place.f.total_cmp(&c2.time_place.f));
		collisions
	}

	pub fn neighbor_collisions(&self) -> Vec<Collision> {
		let mut vec: Vec<Collision> = vec![];
		let n = self.segments.len();
		for i in 0..n {
			let h = (n - 1 + i) % n;
			let j = (n + 1 + i) % n;
			let prev = &self.segments[h];
			let this = &self.segments[i];
			let next = &self.segments[j];
			let cols = three_circle_collision(
				&prev.circle_neg_r(),
				&this.circle_neg_r(),
				&next.circle_neg_r(),
			);
			for col in cols {
				let FloatVec2 { f: t, v: p } = col;
				if t > 0.0 {
					let shrunk = self.shrink_naive(t - f32::EPSILON);
					let sthis = &shrunk.segments[i];
					let snext = &shrunk.segments[j];
					let thisd = (sthis.initial - p).length();
					let nextd = (snext.initial - p).length();
					const LIMIT: f32 = 1.0;
					if thisd < LIMIT && nextd < LIMIT {
						vec.push(Collision {
							time_place: col,
							kind: CollisionType::Neighbors { idx: i },
						});
					}
				}
			}
		}
		vec
	}

	pub fn opposite_collisions(&self) -> Vec<Collision> {
		let mut vec: Vec<Collision> = vec![];
		let n = self.segments.len();
		if n <= 3 {
			return vec![];
		}
		for i in 0..n {
			let first = &self.segments[i];
			let first_r = first.radius();
			for j in i + 2..n {
				if i == 0 && j == n - 1 {
					continue;
				}
				let second = &self.segments[j];
				if first.bend == Bend::Inward && second.bend == Bend::Inward {
					let second_r = second.radius();
					let center_line = second.center - first.center;
					let dist = center_line.length();
					let t = 0.5 * (dist - first_r - second_r);
					if t >= 0.0 {
						let place = first.center + (first_r + t) * center_line.normalize();
						let naive = self.shrink_naive(t + f32::EPSILON);
						let first_naive = naive.segments[i];
						let second_naive = naive.segments[j];
						let first_naive_next = naive.segments[(n + 1 + i) % n];
						let second_naive_next = naive.segments[(n + 1 + j) % n];
						let [fbv, fba, sbv, sba] = [
							angle_counter_clockwise(
								&(first_naive_next.initial - first_naive.center),
								&(place - first_naive.center),
							),
							angle_counter_clockwise(
								&(first_naive_next.initial - first_naive.center),
								&(first_naive.initial - first_naive.center),
							),
							angle_counter_clockwise(
								&(second_naive_next.initial - second_naive.center),
								&(place - second_naive.center),
							),
							angle_counter_clockwise(
								&(second_naive_next.initial - second_naive.center),
								&(second_naive.initial - second_naive.center),
							),
						];
						if fbv < fba && sbv < sba {
							let col = Collision {
								time_place: FloatVec2 { f: t, v: place },
								kind: CollisionType::Opposite { first_idx: i, second_idx: j },
							};

							vec.push(col);
						}
					}
				} else {
					todo!()
				}
			}
		}
		vec
	}

	pub fn max_arc_length(&self) -> f32 {
		self
			.segments
			.windows(2)
			.map(|pair| (pair[1].initial - pair[0].initial).length())
			.reduce(f32::max)
			.unwrap_or(f32::MAX)
	}

	pub fn with_removed(&self, idx: usize) -> ArcPoly {
		let mut clone = self.clone();
		clone.segments.remove(idx);
		clone
	}

	pub fn shrink_naive(&self, amount: f32) -> ArcPoly {
		let n = self.segments.len();
		let mut segs: Vec<Segment> = vec![];
		for j in 0..n {
			let i = (n - 1 + j) % n;
			let (a, b) = (&self.segments[i], &self.segments[j]);
			if a.bend == Bend::Inward && b.bend == Bend::Inward {
				let (mut ca, mut cb) = (a.circle(), b.circle());
				ca.f += amount;
				cb.f += amount;
				let cols = two_circle_collision(&ca, &cb);
				if cols.len() < 2 {
					println!("{}, {}", ca, cb);
					panic!("circles not intersecting")
				}
				segs.push(Segment { initial: cols[1], center: b.center, bend: b.bend });
			} else {
				todo!();
			}
		}

		ArcPoly { segments: segs }
	}

	pub fn from_gen_input(gen_input: &ArcPolyGenInput) -> Self {
		let n = gen_input.n;
		let mut rng = StdRng::seed_from_u64(gen_input.random_seed as u64);
		let mut res = ArcPoly::default();
		let mut pts: Vec<Vec2> = default();
		for i in 0..n {
			pts.push(
				Vec2::new(
					f32::cos(2.0 * PI * (i as f32) / (gen_input.n as f32)),
					f32::sin(2.0 * PI * (i as f32) / (gen_input.n as f32)),
				) * gen_input.r
					+ Vec2::from_array(UnitDisc.sample(&mut rng))
						* gen_input.offset_noise,
			);
		}
		for (i, j) in (0..n).circular_tuple_windows() {
			let (a, b) = (pts[i], pts[j]);
			let absolute_bend = rng.gen_range(
				gen_input.bend_min
					..f32::max(gen_input.bend_min + 0.01, gen_input.bend_max),
			);
			let bend = Bend::Inward;
			let c = circle_center_from_3_points(
				&a,
				&b,
				&(midpoint(&a, &b)
					+ (b - a).rotate(Vec2::NEG_Y)
						* absolute_bend
						* bool_to_sign(bend == Bend::Outward)),
			);
			res.segments.push(Segment { initial: a, center: c, bend: bend });
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
	let n = arc_poly.segments.len();
	let mut j: usize = 0;
	let mut polys: Vec<ArcPoly> = vec![default(), default()];
	for i in 0..n {
		let segment = &arc_poly.segments[i];
		if [first_idx, second_idx].contains(&i) {
			let mut right = segment.clone();
			right.initial = place;
			polys[j].segments.push(segment.clone());
			j = (j + 1) % 2;
			polys[j].segments.push(right);
		} else {
			polys[j].segments.push(segment.clone());
		}
	}
	polys
}

#[derive(Reflect, Resource)]
pub struct ArcPolyGenInput {
	pub random_seed: u32,
	pub n: usize,
	pub r: f32,
	pub offset_noise: f32,
	pub bend_max: f32,
	pub bend_min: f32,
	pub shrink: f32,
}

impl Default for ArcPolyGenInput {
	fn default() -> Self {
		ArcPolyGenInput {
			random_seed: 17,
			n: 13,
			r: 250.0,
			offset_noise: 50.0,
			bend_max: 0.5,
			bend_min: 0.02,
			shrink: 48.5,
		}
	}
}
