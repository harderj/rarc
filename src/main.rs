use std::borrow::BorrowMut;

use bevy::{
	app::{App, Startup, Update},
	core_pipeline::core_2d::Camera2dBundle,
	ecs::system::{Commands, Query},
	gizmos::gizmos::Gizmos,
	prelude::*,
	DefaultPlugins,
};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use rarc::geom::arc_poly::{
	ArcPoly, ArcPolyGenInput, Collision, CollisionType,
};

fn main() {
	App::new()
		.init_resource::<ArcPolyGenInput>()
		.add_plugins(DefaultPlugins)
		.add_plugins(ResourceInspectorPlugin::<ArcPolyGenInput>::new())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(mut commands: Commands, gen_input: ResMut<ArcPolyGenInput>) {
	commands.spawn(Camera2dBundle::default());
	commands.spawn(ArcPoly::from_gen_input(&gen_input));
}

fn update(
	mut gizmos: Gizmos,
	gen_input: ResMut<ArcPolyGenInput>,
	mut arc_poly_query: Query<&mut ArcPoly>,
) {
	let mut arc_poly = arc_poly_query.single_mut();
	if gen_input.is_changed() {
		// TODO: this is probably not the right way to do it
		let borrowed: &mut ArcPoly = arc_poly.borrow_mut();
		*borrowed = ArcPoly::from_gen_input(&gen_input);
	}
	arc_poly.draw(&mut gizmos, &Color::BLUE);
	let shrunk = arc_poly.shrunk(&mut gizmos, gen_input.shrink.max(0.0));
	for sub_poly in shrunk {
		sub_poly.draw(&mut gizmos, &Color::GREEN);
		// for col in sub_poly.future_collisions() {
		// 	if let Collision { kind: CollisionType::Neighbors { .. }, time_place } =
		// 		col
		// 	{
		// 		gizmos.circle_2d(time_place.v, 5.0, Color::WHITE);
		// 	}
		// }
	}
}
