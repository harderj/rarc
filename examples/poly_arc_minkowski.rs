use std::f32::consts::PI;

use bevy::{
	DefaultPlugins,
	app::{App, Startup, Update},
	color::Color,
	core_pipeline::core_2d::Camera2d,
	ecs::{
		resource::Resource,
		system::{Commands, ResMut},
	},
	gizmos::gizmos::Gizmos,
	math::Vec2,
	reflect::Reflect,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use rand::{Rng, SeedableRng, rngs::StdRng};
use rarc::{
	geom::{arc::Arc, arc_graph::ArcGraph, misc::DrawableWithGizmos},
	util::FloatResource,
};

#[derive(Clone, Copy, Resource, Reflect)]
struct CustomResource {
	random_seed: u32,
	vertices: usize,
	poly_radius: f32,
	bend_mean: f32,
	bend_noise: f32,
	offset_noise: f32,
	radius: FloatResource,
	show_original: bool,
	show_minkowski_debug: bool,
	show_minkowski: bool,
	extra_radius: f32,
}

impl Default for CustomResource {
	fn default() -> Self {
		CustomResource {
			random_seed: 0,
			vertices: 3,
			poly_radius: 8.0,
			bend_mean: 2.0,
			bend_noise: 2.0,
			offset_noise: 5.0,
			radius: FloatResource { scale: 10.0, value: 5.0 },
			show_original: true,
			show_minkowski_debug: false,
			show_minkowski: true,
			extra_radius: -5.0,
		}
	}
}

fn main() {
	App::new()
		.add_plugins((DefaultPlugins, PanCamPlugin::default()))
		.init_resource::<CustomResource>()
		.add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
		.add_plugins(ResourceInspectorPlugin::<CustomResource>::default())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn((
		Camera2d::default(),
		PanCam { grab_buttons: vec![], ..Default::default() },
	));
}

fn update(mut gizmos: Gizmos, resource: ResMut<CustomResource>) {
	let arcs = gen_poly(*resource.as_ref());
	if resource.show_original {
		arcs.iter().for_each(|a| a.draw_gizmos(&mut gizmos, Some(Color::BLACK)));
	}
	let radius = resource.radius.get();
	if resource.show_minkowski_debug {
		let sum: ArcGraph =
			arcs.iter().map(|&a| ArcGraph::minkowski_arc(a, radius)).sum();
		sum.draw_gizmos(&mut gizmos, None);
	}
	if resource.show_minkowski {
		let m = ArcGraph::minkowski(arcs, radius);
		m.draw_gizmos(&mut gizmos, Some(Color::WHITE));
	}
}

fn gen_poly(resource: CustomResource) -> Vec<Arc> {
	let mut rng = StdRng::seed_from_u64(resource.random_seed as u64);
	let mut arcs: Vec<Arc> = Vec::new();
	let n = resource.vertices;
	let mut points: Vec<Vec2> = Vec::new();
	for i in 0..n {
		let angle = 2.0 * PI * ((i + 1) as f32) / (n as f32);
		let direction = Vec2::from_angle(angle);
		let offset_angle = 2.0 * PI * rng.random::<f32>();
		let offset = Vec2::from_angle(offset_angle) * resource.offset_noise * 10.0;
		points.push(direction * resource.poly_radius * 30.0 + offset);
	}
	for i in 0..n {
		let (a, b) = (points[i], points[(i + 1) % n]);
		let bend = resource.bend_mean + resource.bend_noise;
		arcs.push(Arc::from_bend_and_endpoints(a, b, bend * 0.02));
	}
	for arc in arcs.iter_mut() {
		arc.radius += resource.extra_radius;
	}
	arcs
}
