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
use rarc::{
	geom::{arc::Arc, circle::Circle, misc::DrawableWithGizmos},
	util::FloatResource,
};

#[derive(Default, Resource, Reflect)]
struct CustomResource {
	arc1: Arc,
	arc2: Arc,
	time: FloatResource,
	show_original: bool,
	show_minkowski: bool,
}

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.init_resource::<CustomResource>()
		.add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
		.add_plugins(ResourceInspectorPlugin::<CustomResource>::default())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(mut commands: Commands, mut resource: ResMut<CustomResource>) {
	commands.spawn(Camera2d::default());
	resource.arc1 =
		Arc { mid: 3.0, span: PI, radius: 130.0, ..Default::default() };
	resource.arc2 =
		Arc { mid: -9.0, span: PI, radius: 150.0, center: Vec2::X * 30.0 };
	resource.show_original = true;
	resource.show_minkowski = true;
}

fn update(mut gizmos: Gizmos, resource: ResMut<CustomResource>) {
	let (arc1, arc2) = (resource.arc1, resource.arc2);
	if resource.show_original {
		[resource.arc1, resource.arc2]
			.map(|a| a.draw_gizmos(&mut gizmos, Color::WHITE));
		arc1
			.intersect(arc2)
			.into_iter()
			.for_each(|p| Circle::new(5.0, p).draw_gizmos(&mut gizmos, Color::WHITE));
	}
	if resource.show_minkowski {
		// todo
	}
}
