use bevy::{
	DefaultPlugins,
	app::{App, Startup, Update},
	core_pipeline::core_2d::Camera2d,
	ecs::system::Commands,
	gizmos::gizmos::Gizmos,
};

fn main() {
	App::new()
		.add_plugins(DefaultPlugins) // .build().disable::<PipelinedRenderingPlugin>())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2d::default());
}

fn update(mut _gizmos: Gizmos) {
	// todo!()
}
