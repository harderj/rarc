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
};
use itertools::Itertools;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, UnitDisc};

use crate::math::{
	bool_to_sign, circle_center_from_3_points, three_circle_collision,
	two_circle_collision, Circle, FloatVec2,
};
use crate::{
	geom::arc_poly::CollisionType::Opposite, math::angle_counter_clockwise,
};

#[derive(Clone, Copy, Display, Reflect, PartialEq)]
pub enum Bend {
	Inward,
	Outward,
}

#[derive(Component, Copy, Reflect, Clone, Display)]
#[display(fmt = "segment({}, {})", initial, bend)]
pub struct Segment {
	pub initial: Vec2,
	pub center: Vec2,
	pub bend: Bend,
}

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

#[derive(Display)]
#[display(fmt = "collision({}, {})", kind, time_place)]
pub struct Collision {
	time_place: FloatVec2,
	kind: CollisionType,
}

#[derive(Display)]
pub enum CollisionType {
	#[display(fmt = "opposite({}, {})", first_idx, second_idx)]
	Opposite { first_idx: usize, second_idx: usize },
	#[display(fmt = "neighbors({})", idx)]
	Neighbors { idx: usize },
}

impl Segment {
	pub fn extreme(&self, next_initial: &Vec2) -> Vec2 {
		0.5 * (self.initial + *next_initial)
			+ 0.5
				* self.outward(next_initial)
				* bool_to_sign(self.bend == Bend::Outward)
	}

	pub fn outward(&self, next_initial: &Vec2) -> Vec2 {
		(*next_initial - self.initial).rotate(Vec2::NEG_Y)
	}

	pub fn ca(&self) -> Vec2 {
		self.initial - self.center
	}

	pub fn cb(&self, b_initial: &Vec2) -> Vec2 {
		*b_initial - self.center
	}

	pub fn radius(&self) -> f32 {
		self.ca().length()
	}

	pub fn angle(&self, next_initial: &Vec2) -> f32 {
		angle_gen(&self.ca(), &self.cb(next_initial), self.bend)
	}

	pub fn angle_a(&self) -> f32 {
		let ca = self.ca();
		f32::atan2(ca.y, ca.x)
	}

	pub fn angle_b(&self, next_initial: &Vec2) -> f32 {
		let cb = self.cb(next_initial);
		f32::atan2(cb.y, cb.x)
	}

	pub fn circle(&self) -> Circle {
		FloatVec2 { v: self.center, f: self.radius() }
	}

	pub fn circle_neg_r(&self) -> Circle {
		FloatVec2 {
			v: self.center,
			f: self.radius() * bool_to_sign(self.bend == Bend::Inward),
		}
	}
}

pub fn angle_gen(ca: &Vec2, cb: &Vec2, bend: Bend) -> f32 {
	if bend == Bend::Outward {
		angle_counter_clockwise(ca, cb)
	} else {
		angle_counter_clockwise(cb, ca)
	}
}

pub fn midpoint(a: &Vec2, b: &Vec2) -> Vec2 {
	0.5 * (*a + *b)
}

pub fn draw_segment(
	a: &Segment,
	b_initial: &Vec2,
	gizmos: &mut Gizmos,
	color: &Color,
) {
	gizmos.circle_2d(a.initial, 2.0, Color::BLACK);
	gizmos.circle_2d(*b_initial, 4.0, Color::GRAY);
	gizmos.arc_2d(
		Vec2::from_array(a.center.into()),
		a.outward(b_initial).angle_between(Vec2::Y)
			+ (a.bend == Bend::Inward).then_some(PI).unwrap_or(0.0),
		a.angle(b_initial),
		a.radius(),
		*color,
	);
}

impl ArcPoly {
	pub fn draw(&self, gizmos: &mut Gizmos, color: &Color) {
		for (i, j) in (0..self.segments.len()).circular_tuple_windows() {
			let (a, b) = (&self.segments[i], &self.segments[j]);
			draw_segment(a, &b.initial, gizmos, color);
		}
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
			let prev = &self.segments[(n + 0 + i) % n];
			let this = &self.segments[(n + 1 + i) % n];
			let next = &self.segments[(n + 2 + i) % n];
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
		let n = self.segments.len();
		if n <= 3 {
			return vec![];
		}
		for i in 0..n {
			let first = &self.segments[i];
			let first_next = &self.segments[(n + 1 + i) % n];
			let first_c = first.center;
			let first_r = first.radius();
			for j in i + 2..n {
				if i == 0 && j == n - 1 {
					continue;
				}
				let second = &self.segments[j];
				let second_next = &self.segments[(n + 1 + j) % n];
				if first.bend == Bend::Inward && second.bend == Bend::Inward {
					let second_c = second.center;
					let second_r = second.radius();
					let center_line = second_c - first_c;
					let dist = center_line.length();
					let t = 0.5 * (dist - first_r - second_r);
					if t >= 0.0 {
						let place = first_c + (first_r + t) * center_line.normalize();
						let naive = self.shrink_naive(t);
						let first_naive = naive.segments[i];
						let second_naive = naive.segments[j];
						let first_naive_next = naive.segments[(n + 1 + i) % n];
						let second_naive_next = naive.segments[(n + 1 + j) % n];
						let first_naive_c = first_naive.center;
						let second_naive_c = second_naive.center;
						let [fbv, fba, sbv, sba] = [
							angle_counter_clockwise(
								&(first_naive_next.initial - first_naive_c),
								&(place - first_naive_c),
							),
							angle_counter_clockwise(
								&(first_naive_next.initial - first_naive_c),
								&(first_naive.initial - first_naive_c),
							),
							angle_counter_clockwise(
								&(second_naive_next.initial - second_naive_c),
								&(place - second_naive_c),
							),
							angle_counter_clockwise(
								&(second_naive_next.initial - second_naive_c),
								&(second_naive.initial - second_naive_c),
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
			.segments
			.windows(2)
			.map(|pair| (pair[1].initial - pair[0].initial).length())
			.reduce(f32::max)
			.unwrap_or(f32::MAX)
	}

	pub fn shrunk(&self, gizmos: &mut Gizmos, amount: f32) -> Vec<ArcPoly> {
		let collisions = self.future_collisions();
		if let Some(c) = collisions.first() {
			let t = c.time_place.f;
			if 0.0 < t && t < amount {
				let shrunk = self.shrink_naive(t + 0.1);
				if self.segments.len() <= 3 {
					return vec![];
				}
				let children = match c.kind {
					CollisionType::Opposite { first_idx: first, second_idx: second } => {
						// println!("opposite");
						split_opposite(shrunk, c.time_place.v, first, second)
					}
					CollisionType::Neighbors { idx: i } => {
						// println!("neighbor");
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

	pub fn with_removed(&self, idx: usize) -> ArcPoly {
		let mut clone = self.clone();
		clone.segments.remove(idx);
		clone
	}

	pub fn shrink_naive(&self, amount: f32) -> ArcPoly {
		let n = self.segments.len();
		let mut segs: Vec<Segment> = vec![];
		for (i, j, k) in (0..n).circular_tuple_windows::<(_, _, _)>() {
			let (a, b, c) = (&self.segments[i], &self.segments[j], &self.segments[k]);
			if a.bend == Bend::Inward && b.bend == Bend::Inward {
				let (mut ca, mut cb) = (a.circle(), b.circle());
				ca.f += amount;
				cb.f += amount;
				let cols = two_circle_collision(&ca, &cb);
				if cols.len() < 2 {
					panic!("circles not intersecting")
				}
				segs.push(Segment { initial: cols[1], center: b.center, bend: b.bend });
			// println!("{}, {}, {}", i, j, k);
			} else {
				todo!();
			}
		}
		ArcPoly { segments: segs }
	}

	pub fn from_gen_input(gen_input: &ArcPolyGenInput) -> Self {
		let mut rng = StdRng::seed_from_u64(gen_input.random_seed as u64);
		let mut res = ArcPoly::default();
		let mut pts: Vec<Vec2> = default();
		for i in 0..gen_input.n {
			pts.push(
				Vec2::new(
					f32::cos(2.0 * PI * (i as f32) / (gen_input.n as f32)),
					f32::sin(2.0 * PI * (i as f32) / (gen_input.n as f32)),
				) * gen_input.r
					+ Vec2::from_array(UnitDisc.sample(&mut rng))
						* gen_input.offset_noise,
			);
		}
		for (i, j) in (0..gen_input.n).circular_tuple_windows() {
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
			res.segments.push(Segment {
				initial: a,
				center: c,
				bend: bend, // if rng.gen_bool(0.5) { absolute_bend } else { -absolute_bend },
			});
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
			random_seed: 0,
			n: 5,
			r: 250.0,
			offset_noise: 50.0,
			bend_max: 0.5,
			bend_min: 0.02,
			shrink: 10.0,
		}
	}
}
