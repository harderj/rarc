use bevy::{
	app::{App, Startup},
	core_pipeline::core_2d::Camera2dBundle,
	ecs::system::Commands,
	gizmos::gizmos::Gizmos,
	prelude::*,
	DefaultPlugins
};
use rarc::math::{two_circle_collision, FloatVec2};

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2dBundle::default());
}

fn gizmo_circle(gizmos: &mut Gizmos, circle: FloatVec2, color: Color) {
	gizmos.circle_2d(circle.v, circle.f, color);
}

fn update(mut gizmos: Gizmos) {
	let c1 = FloatVec2 {
		v: default(),
		f: 200.0,
	};
	let c2 = FloatVec2 {
		v: Vec2::new(-150.0, 0.0),
		f: 200.0,
	};
	gizmo_circle(&mut gizmos, c1, Color::BLUE);
	gizmo_circle(&mut gizmos, c2, Color::GREEN);

	let collisions = two_circle_collision(c1, c2);
	for c in collisions {
		gizmo_circle(
			&mut gizmos,
			FloatVec2 { v: c, f: 20.0 },
			Color::ORANGE_RED
		)
	}
}
