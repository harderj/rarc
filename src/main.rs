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
use rarc::geom::{
	arc::Arc,
	arc_graph::ArcGraph,
	misc::{DrawGizmosOptions, DrawableWithGizmos},
};

#[derive(Clone, Copy, Resource, Reflect)]
struct CustomResource {
	random_seed: u32,
	vertex_count: usize,
	vertex_radius: f32,
	vertex_noise: f32,
	bend_mean: f32,
	bend_noise: f32,
	bend_abs_min: f32,
	joint_offset: f32,
	minkowski_offset: f32,
	show_original: bool,
	show_debug: bool,
	show_minkowski: bool,
}

impl Default for CustomResource {
	fn default() -> Self {
		CustomResource {
			random_seed: 0,
			vertex_count: 11,
			vertex_noise: 12.0,
			vertex_radius: 6.5,
			bend_mean: 0.0,
			bend_noise: 30.0,
			bend_abs_min: 1.0,
			joint_offset: 0.5,
			minkowski_offset: 3.0,
			show_original: true,
			show_debug: false,
			show_minkowski: true,
		}
	}
}

fn main() {
	App::new()
		.add_plugins((DefaultPlugins, PanCamPlugin::default()))
		.init_resource::<CustomResource>()
		.add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
		.add_plugins(ResourceInspectorPlugin::<CustomResource>::new())
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
		arcs.iter().for_each(|a| {
			a.draw_gizmos(&mut gizmos, &DrawGizmosOptions::from_color(Color::BLACK))
		});
	}
	let radius = resource.minkowski_offset.max(0.1) * 10.0;
	if resource.show_debug {
		let sum: ArcGraph =
			arcs.iter().map(|&a| ArcGraph::minkowski_arc(a, radius)).sum();
		sum.draw_gizmos(&mut gizmos, &DrawGizmosOptions::default());
	}
	if resource.show_minkowski {
		let m = ArcGraph::minkowski(arcs, radius);
		m.draw_gizmos(&mut gizmos, &DrawGizmosOptions::from_color(Color::WHITE));
	}
}

fn gen_poly(resource: CustomResource) -> Vec<Arc> {
	let mut rng = StdRng::seed_from_u64(resource.random_seed as u64);
	let mut arcs: Vec<Arc> = Vec::new();
	let n = resource.vertex_count;
	let mut points: Vec<Vec2> = Vec::new();
	for i in 0..n {
		let angle = 2.0 * PI * ((i + 1) as f32) / (n as f32);
		let direction = Vec2::from_angle(angle);
		let offset_angle = 2.0 * PI * rng.random::<f32>();
		let offset = Vec2::from_angle(offset_angle) * resource.vertex_noise * 10.0;
		points.push(direction * resource.vertex_radius * 30.0 + offset);
	}
	for i in 0..n {
		let (a, b) = (points[i], points[(i + 1) % n]);
		let mut bend = resource.bend_mean
			+ (2.0 * rng.random::<f32>() - 1.0) * resource.bend_noise;
		bend += (resource.bend_abs_min.abs() + 0.05) * bend.signum();
		arcs.push(Arc::from_bend_and_endpoints(a, b, bend * 0.02));
	}
	for arc in arcs.iter_mut() {
		arc.radius += resource.joint_offset;
	}
	arcs
}
