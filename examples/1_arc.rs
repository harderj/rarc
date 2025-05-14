use std::f32::consts::FRAC_PI_2;

use bevy::{
	DefaultPlugins,
	app::{App, Startup, Update},
	color::{Alpha, Color},
	core_pipeline::core_2d::Camera2d,
	ecs::system::{Commands, ResMut},
	gizmos::gizmos::Gizmos,
	math::Vec2,
};

use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use rarc::{
	geom::{arc::Arc, circle::Circle, misc::DrawableWithGizmos},
	util::{FloatResource, color_hash},
};

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.init_resource::<FloatResource>()
		.init_resource::<Arc>()
		.add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
		.add_plugins(ResourceInspectorPlugin::<Arc>::default())
		.add_plugins(ResourceInspectorPlugin::<FloatResource>::default())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(
	mut commands: Commands,
	mut arc: ResMut<Arc>,
	mut time: ResMut<FloatResource>,
) {
	commands.spawn(Camera2d::default());
	*arc =
		Arc { span: FRAC_PI_2, mid: FRAC_PI_2, radius: 100.0, center: Vec2::ZERO };
	time.value = 11.0;
	time.scale = 10.0;
}

fn update(mut gizmos: Gizmos, arc: ResMut<Arc>, time: ResMut<FloatResource>) {
	arc.draw_gizmos(&mut gizmos, Color::WHITE);
	let minkowski_disc = arc.minkowski_disc(time.get());
	for i in minkowski_disc.node_indices() {
		let arc = minkowski_disc.node_weight(i).unwrap();
		arc.draw_gizmos(&mut gizmos, color_hash(i.index()));
	}
	for i in minkowski_disc.edge_indices() {
		let (j, k) = minkowski_disc.edge_endpoints(i).unwrap();
		let &p = minkowski_disc.edge_weight(i).unwrap();
		Circle::new(4.0, p)
			.draw_gizmos(&mut gizmos, color_hash(j.index()).with_alpha(0.3));
		Circle::new(6.0, p)
			.draw_gizmos(&mut gizmos, color_hash(k.index()).with_alpha(0.3));
	}
}
