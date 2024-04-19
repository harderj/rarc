use bevy::{
	app::{App, Startup, Update},
	core_pipeline::core_2d::Camera2dBundle,
	ecs::system::{Commands, Query},
	gizmos::gizmos::Gizmos,
	math::Vec2,
	prelude::default,
	render::color::Color,
	text::{Text, TextStyle},
	ui::{node_bundles::TextBundle, Style, Val},
	DefaultPlugins,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rarc::geom::arc::Arc;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.register_type::<Arc>()
		.add_plugins(WorldInspectorPlugin::new())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2dBundle::default());
	commands.spawn(Arc {
		a: Vec2::new(0.0, 0.0),
		b: Vec2::new(100.0, 100.0),
		s: 100.0,
	});
	commands.spawn(
		TextBundle::from_section(
			"",
			TextStyle {
				font_size: 18.0,
				color: Color::WHITE,
				..default()
			},
		)
		.with_style(Style {
			position_type: bevy::ui::PositionType::Absolute,
			bottom: Val::Px(10.0),
			left: Val::Px(10.0),
			..default()
		}),
	);
}

fn update(
	mut gizmos: Gizmos,
	arc_query: Query<&Arc>,
	mut text: Query<&mut Text>,
) {
	let mut text = text.single_mut();
	let text = &mut text.sections[0].value;

	let arc = arc_query.single();
	arc.draw(&mut gizmos);
	debug_arc_text(arc, text);
}

fn debug_arc_text(arc: &Arc, text: &mut String) {
	*text = "".to_string();
	text.push_str(&format!("A: {}\n", arc.a));
	text.push_str(&format!("B: {}\n", arc.b));
	text.push_str(&format!("Extreme: {}\n", arc.extreme()));
	text.push_str(&format!("Angle: {}\n", arc.angle()));
	text.push_str(&format!("AngleA: {}\n", arc.angle_a()));
	text.push_str(&format!("CA: {}\n", arc.ca()));
	text.push_str(&format!("CB: {}\n", arc.cb()));
	text.push_str(&format!("Center: {}\n", arc.center()));
	text.push_str(&format!("Radius: {}\n", arc.radius()));
}
