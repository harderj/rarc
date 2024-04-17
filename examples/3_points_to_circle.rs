use std::f32::consts::PI;

use bevy::{
	DefaultPlugins,
	app::{App, Startup},
	color::palettes::css::*,
	ecs::system::Commands,
	gizmos::gizmos::Gizmos,
	prelude::*,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use rarc::math::circle_center_from_3_points;

const CIRCLE_COLORS: [Color; 4] = [
	Color::Srgba(RED),
	Color::Srgba(ORANGE),
	Color::Srgba(YELLOW),
	Color::Srgba(YELLOW_GREEN),
];

const CIRCLE_RESOLUTION: u32 = 128;

#[derive(Resource, Reflect, Default, Deref, DerefMut)]
#[reflect(Resource)]
struct Vec2Triple([Vec2; 3]);

fn main() {
	App::new()
		.init_resource::<Vec2Triple>()
		.add_plugins(DefaultPlugins)
		.add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
		.add_plugins(ResourceInspectorPlugin::<Vec2Triple>::new())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(mut commands: Commands, mut triple: ResMut<Vec2Triple>) {
	commands.spawn(Camera2d::default());
	for (i, point) in triple.iter_mut().enumerate() {
		*point = Vec2::from_angle(2.0 * PI * (i as f32) / 3.0) * 100.0;
	}
}

fn update(mut gizmos: Gizmos, triple: Res<Vec2Triple>) {
	for (point, color) in triple.iter().zip(CIRCLE_COLORS) {
		gizmos.circle_2d(*point, 4.0, color).resolution(CIRCLE_RESOLUTION);
	}

	let center = circle_center_from_3_points(triple[0], triple[1], triple[2]);
	gizmos.circle_2d(center, 4.0, Color::WHITE).resolution(CIRCLE_RESOLUTION);
	gizmos
		.circle_2d(center, (center - triple[0]).length(), Color::Srgba(GRAY))
		.resolution(CIRCLE_RESOLUTION);
}
