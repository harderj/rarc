use std::f32::consts::PI;

use crate::math::{
	circle_center_from_3_points, two_circle_collision, FloatVec2,
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

	fn adjust_neighbors(&mut self, previous: Arc, next: Arc) {
		let self_circle = FloatVec2 {
			v: self.center(),
			f: self.radius(),
		};
		let previous_circle = FloatVec2 {
			v: previous.center(),
			f: previous.radius(),
		};
		let next_circle = FloatVec2 {
			v: next.center(),
			f: next.radius(),
		};
		let cols_previous = two_circle_collision(previous_circle, self_circle);
		let cols_next = two_circle_collision(self_circle, next_circle);
		// println!("previous: {}, self: {}", previous_circle, self_circle);
		if cols_previous.len() > 1 {
			self.a = cols_previous[0];
		}
		if cols_next.len() > 1 {
			self.b = cols_next[0];
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
		gizmos.circle_2d(Vec2::from_array(self.a.into()), 2.0, Color::GRAY);
		gizmos.circle_2d(
			Vec2::from_array(self.b.into()),
			4.0,
			Color::DARK_GRAY,
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
	pub shrink: f32,
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

	pub fn shrunk(&self) -> Vec<ArcPoly> {
		let mut arcs = self.original.clone();
		let n = arcs.len();

		for arc in arcs.iter_mut() {
			arc.shrink(self.shrink);
		}

		for i in 0..arcs.len() {
			let previous = arcs[(n + i - 1) % n].clone();
			let next = arcs[(i + 1) % n].clone();
			arcs[i].adjust_neighbors(previous, next);
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
