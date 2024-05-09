use bevy::{
	app::{App, Startup},
	core_pipeline::core_2d::Camera2dBundle,
	ecs::system::Commands,
	gizmos::gizmos::Gizmos,
	prelude::*,
	DefaultPlugins,
};
use bevy_inspector_egui::quick::{
	ResourceInspectorPlugin, WorldInspectorPlugin,
};

use rarc::{
	math::{three_circle_collision, two_circle_collision, Circle, FloatVec2},
	util::{gizmo_circle, TimeResource},
};

fn main() {
	App::new()
		.init_resource::<TimeResource>()
		.register_type::<FloatVec2>()
		.add_plugins(DefaultPlugins)
		.add_plugins(ResourceInspectorPlugin::<TimeResource>::new())
		.add_plugins(WorldInspectorPlugin::new())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2dBundle::default());
	for c in [
		FloatVec2 { v: Vec2::new(0.0, 100.0), f: 150.0 },
		FloatVec2 { v: Vec2::new(-100.0, -50.0), f: 70.0 },
		FloatVec2 { v: Vec2::new(100.0, -50.0), f: 60.0 },
	] {
		commands.spawn(c);
	}
}

const CIRCLE_COLORS: [Color; 4] =
	[Color::RED, Color::ORANGE, Color::YELLOW, Color::YELLOW_GREEN];

const COLLISION_COLORS: [Color; 2] = [Color::BLACK, Color::WHITE];

fn update(
	mut gizmos: Gizmos,
	time_resource: Res<TimeResource>,
	circles: Query<&mut Circle>,
) {
	let t = FloatVec2 {
		v: Vec2::default(),
		f: time_resource.time * time_resource.speed,
	};

	for (c, color) in circles.iter().zip(CIRCLE_COLORS) {
		gizmo_circle(&mut gizmos, *c + t, color);
	}

	let mut two_collisions: Vec<(Vec2, Color)> = Vec::default();
	for [c1, c2] in circles.iter_combinations() {
		let collisions = two_circle_collision(&(*c1 + t), &(*c2 + t));
		let mut colored: Vec<(Vec2, Color)> =
			collisions.into_iter().zip(COLLISION_COLORS).collect();
		two_collisions.append(&mut colored);
	}

	for (center, color) in two_collisions {
		gizmo_circle(&mut gizmos, FloatVec2 { v: center, f: 4.0 }, color)
	}

	let mut three_collisions: Vec<FloatVec2> = Vec::default();
	for [c1, c2, c3] in circles.iter_combinations() {
		three_circle_collision(c1, c2, c3).map(|c| three_collisions.push(c));
	}

	for c in three_collisions {
		gizmo_circle(&mut gizmos, FloatVec2 { f: 5.0, v: c.v }, Color::BLUE);
		gizmo_circle(&mut gizmos, c - t, Color::GREEN.with_a(0.3));
	}
}
