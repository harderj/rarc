use std::borrow::BorrowMut;

use bevy::{
	app::{App, Startup, Update},
	core_pipeline::core_2d::Camera2dBundle,
	ecs::system::{Commands, Query},
	gizmos::gizmos::Gizmos,
	prelude::*,
	DefaultPlugins
};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use rarc::geom::arc::{Arc, ArcPoly, ArcPolyGenInput};

fn main() {
	App::new()
		.init_resource::<ArcPolyGenInput>()
		.add_plugins(DefaultPlugins)
		.register_type::<Arc>()
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
	mut arc_poly: Query<&mut ArcPoly>
) {
	if gen_input.is_changed(){
		// TODO: this is probably not the right way to do it
		let mut single = arc_poly.single_mut();
		let borrowed: &mut ArcPoly = single.borrow_mut();
		*borrowed = ArcPoly::from_gen_input(&gen_input);
	}
	arc_poly.single().draw(&mut gizmos);
}
