use std::f32::consts::FRAC_PI_2;

use bevy::{
	DefaultPlugins,
	app::{App, Startup, Update},
	ecs::system::Commands,
	gizmos::gizmos::Gizmos,
	prelude::*,
};

use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use rarc::geom::arc::Arc;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.init_resource::<Arc>()
		.add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
		.add_plugins(ResourceInspectorPlugin::<Arc>::default())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(mut commands: Commands, mut arc: ResMut<Arc>) {
	commands.spawn(Camera2d::default());
	*arc =
		Arc { span: FRAC_PI_2, mid: FRAC_PI_2, radius: 100.0, center: Vec2::ZERO };
}

fn update(mut gizmos: Gizmos, arc: ResMut<Arc>) {
	arc.draw(&mut gizmos, Color::WHITE);
}
