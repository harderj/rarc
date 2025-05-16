use std::f32::consts::FRAC_PI_2;

use bevy::{
	DefaultPlugins,
	app::{App, Startup, Update},
	core_pipeline::core_2d::Camera2d,
	ecs::{
		resource::Resource,
		system::{Commands, ResMut},
	},
	gizmos::gizmos::Gizmos,
	reflect::Reflect,
};

use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use rarc::{
	geom::{
		arc::Arc,
		arc_graph::ArcGraph,
		misc::{DrawGizmosOptions, DrawableWithGizmos},
	},
	util::FloatResource,
};

#[derive(Default, Resource, Reflect)]
struct CustomResource {
	arc: Arc,
	time: FloatResource,
	show_orignal_arc: bool,
	show_minkowski: bool,
}

fn main() {
	App::new()
		.add_plugins((DefaultPlugins, PanCamPlugin::default()))
		.init_resource::<CustomResource>()
		.add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
		.add_plugins(ResourceInspectorPlugin::<CustomResource>::default())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(mut commands: Commands, mut resource: ResMut<CustomResource>) {
	commands.spawn((
		Camera2d::default(),
		PanCam { grab_buttons: vec![], ..Default::default() },
	));
	resource.arc = Arc { span: FRAC_PI_2, radius: 80.0, ..Default::default() };
	resource.time = FloatResource { scale: 10.0, value: 11.0 };
	resource.show_orignal_arc = true;
	resource.show_minkowski = true;
}

fn update(mut gizmos: Gizmos, resource: ResMut<CustomResource>) {
	let arc = resource.arc;
	if resource.show_orignal_arc {
		arc.draw_gizmos(&mut gizmos, &DrawGizmosOptions::default());
	}
	if resource.show_minkowski {
		ArcGraph::minkowski_arc(arc, resource.time.get())
			.draw_gizmos(&mut gizmos, &DrawGizmosOptions::default());
	}
}
