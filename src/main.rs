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
use rarc::geom::arc_poly::{ArcPoly, ArcPolyGenInput};

fn main() {
	println!("hey");
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
	let shrink_amount = gen_input.shrink.max(0.0);
	// let p_shrunk_naive = arc_poly.shrink_naive(shrink_amount, &mut gizmos);
	// if let Some(shrunk_naive) = p_shrunk_naive {
	// 	shrunk_naive.draw(&mut gizmos, &Color::WHITE)
	// }
	let shrunk = arc_poly.shrunk(&mut gizmos, shrink_amount);
	for sub_poly in shrunk {
		sub_poly.draw(&mut gizmos, &Color::GREEN);
	}
	// panic!();
}
