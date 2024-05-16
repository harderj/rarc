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
	circle_center_from_3_points, three_circle_collision, two_circle_collision,
	Circle, FloatVec2,
};
use crate::{
	geom::arc_poly::CollisionType::Opposite, math::angle_counter_clockwise,
};

#[derive(Component, Reflect, Default, Clone, Copy, Display)]
#[display(fmt = "segment({}, {})", initial, bend)]
pub struct Segment {
	pub initial: Vec2,
	pub bend: f32,
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

pub fn outward(a: &Segment, b_initial: &Vec2) -> Vec2 {
	(*b_initial - a.initial).rotate(Vec2::NEG_Y)
}

pub fn extreme(a: &Segment, b_initial: &Vec2) -> Vec2 {
	0.5 * (a.initial + *b_initial) + 0.5 * outward(a, b_initial) * a.bend
}

pub fn center(a: &Segment, b_initial: &Vec2) -> Vec2 {
	circle_center_from_3_points(&a.initial, b_initial, &extreme(a, b_initial))
}

pub fn ca(a: &Segment, b_initial: &Vec2) -> Vec2 {
	a.initial - center(a, b_initial)
}

pub fn cb(a: &Segment, b_initial: &Vec2) -> Vec2 {
	*b_initial - center(a, b_initial)
}

pub fn radius(a: &Segment, b_initial: &Vec2) -> f32 {
	ca(a, b_initial).length()
}

pub fn angle(a: &Segment, b_initial: &Vec2) -> f32 {
	angle_gen(&ca(a, b_initial), &cb(a, b_initial), a.bend)
}

pub fn angle_gen(ca: &Vec2, cb: &Vec2, bend: f32) -> f32 {
	if bend > 0.0 {
		angle_counter_clockwise(ca, cb)
	} else {
		angle_counter_clockwise(cb, ca)
	}
}

pub fn midpoint(a: &Vec2, b: &Vec2) -> Vec2 {
	0.5 * (*a + *b)
}

pub fn angle_a(a: &Segment, b_initial: &Vec2) -> f32 {
	let ca = ca(a, b_initial);
	f32::atan2(ca.y, ca.x)
}

pub fn angle_b(a: &Segment, b_initial: &Vec2) -> f32 {
	let cb = cb(a, b_initial);
	f32::atan2(cb.y, cb.x)
}

pub fn circle(a: &Segment, b_initial: &Vec2) -> Circle {
	FloatVec2 { v: center(a, b_initial), f: radius(a, b_initial) }
}

pub fn circle_neg_r(a: &Segment, b_initial: &Vec2) -> Circle {
	FloatVec2 {
		v: center(a, b_initial),
		f: -radius(a, b_initial) * f32::signum(a.bend),
	}
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
		Vec2::from_array(center(a, b_initial).into()),
		outward(a, b_initial).angle_between(Vec2::Y)
			+ (a.bend < 0.0).then_some(PI).unwrap_or(0.0),
		angle(a, b_initial),
		radius(a, b_initial),
		*color,
	);
}

impl ArcPoly {
	pub fn draw(&self, gizmos: &mut Gizmos, color: &Color) {
		for w in self.segments.windows(2) {
			let (a, b) = (&w[0], &w[1]);
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
			let prev = &self.segments[(n - 1 + i) % n];
			let this = &self.segments[(n + 0 + i) % n];
			let next = &self.segments[(n + 1 + i) % n];
			let last = &self.segments[(n + 2 + i) % n];
			let pcol = three_circle_collision(
				&circle_neg_r(prev, &this.initial),
				&circle_neg_r(this, &next.initial),
				&circle_neg_r(next, &last.initial),
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
			let first_c = center(first, &first_next.initial);
			let first_r = radius(first, &first_next.initial);
			for j in i + 2..n {
				if i == 0 && j == n - 1 {
					continue;
				}
				let second = &self.segments[j];
				let second_next = &self.segments[(n + 1 + j) % n];
				if first.bend < 0.0 && second.bend < 0.0 {
					let second_c = center(second, &second_next.initial);
					let second_r = radius(second, &second_next.initial);
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
						let first_naive_c = center(&first_naive, &first_naive_next.initial);
						let second_naive_c =
							center(&second_naive, &second_naive_next.initial);
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
				let shrunk = self.shrink_naive(t);
				if self.segments.len() <= 3 {
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

	pub fn shrink_naive(&self, shrink: f32) -> ArcPoly {
		let n = self.segments.len();
		todo!()
	}

	pub fn from_gen_input(gen_input: &ArcPolyGenInput) -> Self {
		let mut rng = StdRng::seed_from_u64(gen_input.random_seed as u64);
		let mut res = ArcPoly::default();
		for i in 0..gen_input.n {
			let vertex = Vec2::new(
				f32::cos(2.0 * PI * (i as f32) / (gen_input.n as f32)),
				f32::sin(2.0 * PI * (i as f32) / (gen_input.n as f32)),
			) * gen_input.r
				+ Vec2::from_array(UnitDisc.sample(&mut rng)) * gen_input.offset_noise;
			let absolute_bend = rng.gen_range(
				gen_input.bend_min
					..f32::max(gen_input.bend_min + 0.01, gen_input.bend_max),
			);
			res.segments.push(Segment {
				initial: vertex,
				bend: -absolute_bend, // if rng.gen_bool(0.5) { absolute_bend } else { -absolute_bend },
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
		let segment = arc_poly.segments[i].clone();
		if [first_idx, second_idx].contains(&i) {
			let mut left = segment.clone();
			let mut right = segment.clone();
			// left.set_b_keeping_center(place);
			// right.set_a_keeping_center(place);
			polys[j].segments.push(left);
			j = (j + 1) % 2;
			polys[j].segments.push(right);
		} else {
			polys[j].segments.push(segment);
		}
	}
	// polys
	todo!()
}

#[derive(Reflect, Resource)]
pub struct ArcPolyGenInput {
	pub random_seed: u32,
	pub n: i32,
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
			n: 3,
			r: 340.5,
			offset_noise: 74.1,
			bend_max: 0.5,
			bend_min: 0.02,
			shrink: 101.8,
		}
	}
}
