use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_vector_shapes::{painter::ShapePainter, shapes::DiscPainter, Shape2dPlugin};
use ndarray::{arr2, Axis};
use ndarray_linalg::solve::Determinant;
use std::f32::consts::PI;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins(Shape2dPlugin::default())
		.add_plugins(WorldInspectorPlugin::new())
		.add_systems(Startup, setup)
		.add_systems(Update, update)
		.run();
}

#[allow(dead_code)]
struct Arc {
	a: Vec2,
	b: Vec2,
	s: f32, // arc height (positive is right of A->B)
}

#[allow(dead_code)]
impl Arc {
	fn ab(&self) -> Vec2 {
		self.b - self.a
	}
	fn center(&self) -> Vec2 {
		let f = self.ab().normalize().rotate(Vec2 { x: 0.0, y: -self.s }) + 0.5 * (self.a + self.b);
		circle_from_3_points(self.a, self.b, f).0
	}
	fn radius(&self) -> f32 {
		(self.center() - self.a).length()
	}
	fn draw_gizmos(&self, gizmos: &mut Gizmos, painter: &mut ShapePainter) {
		gizmos.circle_2d(self.a, 2.0, Color::GRAY);
		gizmos.circle_2d(self.b, 4.0, Color::DARK_GRAY);
		gizmos.circle_2d(self.center(), 6.0, Color::BLUE);
		// painter.hollow = true;
		// painter.thickness = 0.4;
		// painter.color = Color::DARK_GREEN;
		// painter.arc(200.0, 0.0, PI * 0.8);
	}
}

fn setup(mut commands: Commands) {
	commands.spawn(Camera2dBundle::default());
}

fn update(
	mut gizmos: Gizmos,
	mut painter: ShapePainter
) {
	let arc = Arc {
		a: Vec2 { x: 0.0, y: 0.0 },
		b: Vec2 { x: 100.0, y: 100.0 },
		s: 100.0,
	};

	arc.draw_gizmos(&mut gizmos, &mut painter);	
}

#[allow(dead_code)]
fn draw_cursor_gizmos(
	window: &Window,
	camera: &Camera,
	camera_transform: &GlobalTransform,
	gizmos: &mut Gizmos,
) {
	let Some(cursor_position) = window.cursor_position() else {
		return;
	};

	// Calculate a world position based on the cursor's position.
	let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
		return;
	};

	gizmos.circle_2d(point, 10.0, Color::RED);
}

fn circle_from_3_points(p1: Vec2, p2: Vec2, p3: Vec2) -> (Vec2, f32) {
	let m = arr2(&[
		[p1.length_squared(), p1.x, p1.y, 1.0],
		[p2.length_squared(), p2.x, p2.y, 1.0],
		[p3.length_squared(), p3.x, p3.y, 1.0],
	]);
	let m1 = m.select(Axis(1), &[1, 2, 3]).det().unwrap_or(0.0);
	let m2 = m.select(Axis(1), &[0, 2, 3]).det().unwrap_or(0.0);
	let m3 = m.select(Axis(1), &[0, 1, 3]).det().unwrap_or(0.0);
	let m4 = m.select(Axis(1), &[0, 1, 2]).det().unwrap_or(0.0);
	let c = Vec2 { x: m2, y: -m3 } * 0.5 / m1;
	let r = (c.length_squared() + m4 / m1).sqrt();
	(c, r)
}
