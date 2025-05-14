use std::f32::consts::FRAC_PI_2;

use derive_more::{Deref, DerefMut};

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
use rarc::geom::{arc::Arc, misc::DrawableWithGizmos};

#[derive(Default, Deref, DerefMut, Reflect, Resource)]
struct TimeResource(f32);

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.init_resource::<TimeResource>()
		.init_resource::<Arc>()
		.add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
		.add_plugins(ResourceInspectorPlugin::<Arc>::default())
		.add_plugins(ResourceInspectorPlugin::<TimeResource>::default())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(
	mut commands: Commands,
	mut arc: ResMut<Arc>,
	mut time: ResMut<TimeResource>,
) {
	commands.spawn(Camera2d::default());
	*arc =
		Arc { span: FRAC_PI_2, mid: FRAC_PI_2, radius: 100.0, center: Vec2::ZERO };
	**time = 110.0;
}

fn update(mut gizmos: Gizmos, arc: ResMut<Arc>, time: ResMut<TimeResource>) {
	arc.draw_gizmos(&mut gizmos, Color::WHITE);
	for &x in arc.minkowski_disc(**time).node_weights() {
		x.draw_gizmos(&mut gizmos, Color::Srgba(GREEN));
	}
}
