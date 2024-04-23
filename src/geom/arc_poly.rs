use std::f32::consts::PI;

use bevy::{ecs::{component::Component, system::Resource}, gizmos::gizmos::Gizmos, math::Vec2, reflect::Reflect, render::color::Color};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, UnitDisc};

use super::arc::Arc;



#[derive(Component, Reflect, Default, Clone)]
pub struct ArcPoly {
	pub original: Vec<Arc>,
	pub shrink: f32
}

impl ArcPoly {
	pub fn draw(&self, gizmos: &mut Gizmos, already_shrunk: bool) {
		for arc in &self.original {
			arc.draw(
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