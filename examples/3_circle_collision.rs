use bevy::{
	DefaultPlugins,
	app::{App, Startup, Update},
	color::{Alpha, Color, palettes::css::*},
	core_pipeline::core_2d::Camera2d,
	ecs::system::{Commands, Query, Res},
	gizmos::gizmos::Gizmos,
	math::{Vec2, vec2},
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::{
	ResourceInspectorPlugin, WorldInspectorPlugin,
};

use rarc::{geom::circle::Circle, util::FloatResource};

fn main() {
	App::new()
		.init_resource::<FloatResource>()
		.register_type::<Circle>()
		.add_plugins(DefaultPlugins)
		.add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
		.add_plugins(ResourceInspectorPlugin::<FloatResource>::new())
		.add_plugins(WorldInspectorPlugin::new())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2d::default());
	for c in [
		Circle { radius: 150.0, center: vec2(0.0, 100.0) },
		Circle { radius: 70.0, center: vec2(-100.0, -50.0) },
		Circle { radius: 60.0, center: vec2(100.0, -50.0) },
	] {
		commands.spawn(c);
	}
}

const CIRCLE_COLORS: [Color; 4] = [
	Color::Srgba(RED),
	Color::Srgba(ORANGE),
	Color::Srgba(YELLOW),
	Color::Srgba(LIME),
];

const COLLISION_COLORS: [Color; 2] = [Color::BLACK, Color::WHITE];

const CIRCLE_RESOLUTION: u32 = 128;

fn update(
	mut gizmos: Gizmos,
	time_resource: Res<FloatResource>,
	circles: Query<&mut Circle>,
) {
	let offset = time_resource.value * time_resource.scale;
	let offset_ = Circle { radius: offset, center: Vec2::ZERO };

	for (Circle { radius: t, center: c }, color) in
		circles.iter().zip(CIRCLE_COLORS)
	{
		gizmos.circle_2d(*c, offset + t, color).resolution(CIRCLE_RESOLUTION);
	}

	let mut two_collisions: Vec<(Vec2, Color)> = Vec::default();
	for [c1, c2] in circles.iter_combinations() {
		let collisions = (*c1 + offset_).intersect(*c2 + offset_);
		let mut colored: Vec<(Vec2, Color)> =
			collisions.into_iter().zip(COLLISION_COLORS).collect();
		two_collisions.append(&mut colored);
	}

	for (center, color) in two_collisions {
		gizmos.circle_2d(center, 4.0, color).resolution(CIRCLE_RESOLUTION);
	}

	let mut three_collisions: Vec<Circle> = Vec::default();
	for [c1, c2, c3] in circles.iter_combinations() {
		three_collisions.append(&mut Circle::three_circle_tangent(*c1, *c2, *c3));
	}

	for Circle { radius: t, center: p } in three_collisions {
		gizmos.circle_2d(p, 5.0, Color::Srgba(BLUE)).resolution(CIRCLE_RESOLUTION);
		gizmos
			.circle_2d(p, offset - t, Color::Srgba(GREEN).with_alpha(0.3))
			.resolution(CIRCLE_RESOLUTION);
	}
}
