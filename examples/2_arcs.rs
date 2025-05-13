use bevy::{
	DefaultPlugins,
	app::{App, Startup, Update},
	ecs::system::Commands,
	gizmos::gizmos::Gizmos,
	prelude::*,
};

use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use rarc::geom::{arc::Arc, misc::DrawableWithGizmos};

#[derive(Resource, Reflect)]
struct Input {
	x: Vec2,
	y: Vec2,
	z: Vec2,
	b1: f32,
	b2: f32,
	t: f32,
}

impl Default for Input {
	fn default() -> Self {
		Input {
			x: Vec2 { x: -150.0, y: 100.0 },
			y: Vec2 { x: -50.0, y: -100.0 },
			z: Vec2 { x: 140.0, y: 80.0 },
			b1: 10.0,
			b2: 8.0,
			t: 5.0,
		}
	}
}

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.init_resource::<Input>()
		.add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
		.add_plugins(ResourceInspectorPlugin::<Input>::default())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2d::default());
}

fn update(mut gizmos: Gizmos, input: ResMut<Input>) {
	static BEND_SCALE: f32 = 0.02;
	let orig: [Arc; 2] = [
		Arc::from_a_b_bend(input.x, input.y, input.b1 * BEND_SCALE),
		Arc::from_a_b_bend(input.y, input.z, input.b2 * BEND_SCALE),
	];
	orig.map(|a| a.draw_gizmos(&mut gizmos, Color::WHITE));

	// let f = input.t * 5.0;
	// let newr = orig.clone();
	// newr.map(|a| a.draw(&mut gizmos, Color::WHITE));
}
