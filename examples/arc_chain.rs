use bevy::{
	app::{App, Startup, Update},
	core_pipeline::core_2d::Camera2dBundle,
	ecs::system::Commands,
	DefaultPlugins,
};
use bevy_inspector_egui::quick::{
	ResourceInspectorPlugin, WorldInspectorPlugin,
};
use rarc::{math::FloatVec2, util::TimeResource};

fn main() {
	App::new()
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

	// let fvs = vec![
	// 	FloatVec2 { v: Vec2::new(100.0, 0.0), f: 0.8 },
	// 	FloatVec2 { v: Vec2::new(-150.0, 0.0), f: 0.5 },
	// 	FloatVec2 { v: Vec2::new(-100.0, -150.0), f: -0.2 },
	// 	FloatVec2 { v: Vec2::new(50.0, -100.0), f: -0.3 },
	// ];
	// commands.spawn(fvs);
}

fn update() {
	// let test_arc =
	// test_arc.draw(&mut gizmos, Color::WHITE);
	// let r = test_arc.radius();
	// let c = test_arc.center();
	// let ang_a = test_arc.angle_a();
	// let ang_b = test_arc.angle_b();
	// gizmos.circle_2d(
	// 	c + vec2(ang_a.cos(), ang_a.sin()) * r,
	// 	6.0,
	// 	Color::RED.with_a(0.3),
	// );
	// gizmos.circle_2d(
	// 	c + vec2(ang_b.cos(), ang_b.sin()) * r,
	// 	6.0,
	// 	Color::RED.with_a(0.3),
	// );

	// let a = test_arc.a;
	// let b = test_arc.b;
	// let bend = test_arc.bend;
	// let new_ang_a = ang_a + arc_chain_query.single().chain[1].bend;
	// let new_a = c + vec2(new_ang_a.cos(), new_ang_a.sin()) * r;
	// let alpha = 0.5 * Arc::angle_gen(new_a - c, b - c, bend);
	// let calc_bend = 2.0 * (1.0 - alpha.cos()) * r / (new_a - b).length()
	// 	* test_arc.bend.signum();

	// let mut test_arc_bend = test_arc.clone();
	// test_arc_bend.a = new_a;
	// test_arc_bend.bend = calc_bend;
	// test_arc_bend.draw(&mut gizmos, Color::GREEN);
}
