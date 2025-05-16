use std::f32::consts::PI;

use bevy::{
	DefaultPlugins,
	app::{App, Startup, Update},
	color::{Color, palettes::css::*},
	core_pipeline::core_2d::Camera2d,
	ecs::system::{Commands, Res, ResMut},
	gizmos::gizmos::Gizmos,
	math::Vec2,
	prelude::{ReflectResource, Resource},
	reflect::Reflect,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use derive_more::{Deref, DerefMut};

use rarc::geom::{
	circle::Circle,
	misc::{DrawGizmosOptions, DrawableWithGizmos},
};

const CIRCLE_COLORS: [Color; 4] = [
	Color::Srgba(RED),
	Color::Srgba(ORANGE),
	Color::Srgba(YELLOW),
	Color::Srgba(YELLOW_GREEN),
];

#[derive(Resource, Reflect, Default, Deref, DerefMut)]
#[reflect(Resource)]
struct Vec2Triple([Vec2; 3]);

fn main() {
	App::new()
		.init_resource::<Vec2Triple>()
		.add_plugins((DefaultPlugins, PanCamPlugin::default()))
		.add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
		.add_plugins(ResourceInspectorPlugin::<Vec2Triple>::new())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(mut commands: Commands, mut triple: ResMut<Vec2Triple>) {
	commands.spawn((
		Camera2d::default(),
		PanCam { grab_buttons: vec![], ..Default::default() },
	));
	for (i, point) in triple.iter_mut().enumerate() {
		*point = Vec2::from_angle(2.0 * PI * (i as f32) / 3.0) * 100.0;
	}
}

fn update(mut gizmos: Gizmos, triple: Res<Vec2Triple>) {
	for (point, color) in triple.iter().zip(CIRCLE_COLORS) {
		Circle { radius: 4.0, center: *point }
			.draw_gizmos(&mut gizmos, &DrawGizmosOptions::from_color(color));
	}

	let circle = Circle::from_3_points(triple[0], triple[1], triple[2]);
	Circle { radius: 4.0, center: circle.center }
		.draw_gizmos(&mut gizmos, &DrawGizmosOptions::default());
	circle.draw_gizmos(
		&mut gizmos,
		&DrawGizmosOptions::from_color(Color::Srgba(GRAY)),
	);
}
