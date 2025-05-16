use std::f32::consts::PI;

use bevy::{
	DefaultPlugins,
	app::{App, Startup, Update},
	color::{Color, palettes::css::GREEN},
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
use rarc::{
	geom::{
		arc::Arc,
		arc_graph::ArcGraph,
		circle::Circle,
		misc::{DrawGizmosOptions, DrawableWithGizmos},
	},
	util::FloatResource,
};

#[derive(Default, Resource, Reflect)]
struct CustomResource {
	arc1: Arc,
	arc2: Arc,
	time: FloatResource,
	show_original: bool,
	show_minkowski: bool,
	show_intersections: bool,
	show_sum: bool,
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

fn setup(mut commands: Commands, mut resource: ResMut<CustomResource>) {
	commands.spawn((
		Camera2d::default(),
		PanCam { grab_buttons: vec![], ..Default::default() },
	));
	resource.arc1 =
		Arc { mid: 3.0, span: PI, radius: 130.0, ..Default::default() };
	resource.arc2 =
		Arc { mid: -9.0, span: PI, radius: 150.0, center: Vec2::X * 30.0 };
	resource.time = FloatResource { scale: 10.0, value: 5.0 };
	resource.show_original = true;
	resource.show_minkowski = false;
	resource.show_intersections = true;
	resource.show_sum = true;
}

fn update(mut gizmos: Gizmos, resource: ResMut<CustomResource>) {
	let (arc1, arc2) = (resource.arc1, resource.arc2);
	if resource.show_original {
		[arc1, arc2]
			.map(|a| a.draw_gizmos(&mut gizmos, &DrawGizmosOptions::default()));
		arc1.intersect(arc2).into_iter().for_each(|p| {
			Circle::new(5.0, p)
				.draw_gizmos(&mut gizmos, &DrawGizmosOptions::default())
		});
	}
	let ms =
		[arc1, arc2].map(|a| ArcGraph::minkowski_arc(a, resource.time.get()));
	if resource.show_minkowski {
		ms.iter().for_each(|m| {
			m.draw_gizmos(
				&mut gizmos,
				&DrawGizmosOptions::from_color(Color::Srgba(GREEN)),
			)
		});
	}
	let [m1, m2] = &ms;
	if resource.show_intersections {
		for (_, _, p) in m1.intersect(&m2) {
			Circle::new(9.0, p)
				.draw_gizmos(&mut gizmos, &DrawGizmosOptions::default());
		}
	}
	if resource.show_sum {
		let m = m1.clone() + m2.clone();
		m.draw_gizmos(&mut gizmos, &DrawGizmosOptions::default());
	}
}
