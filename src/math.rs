use bevy::math::{Mat3, Vec2, Vec3};

pub fn circle_center_from_3_points(p1: Vec2, p2: Vec2, p3: Vec2) -> Vec2 {
	let c1 = Vec3::new(p1.length_squared(), p2.length_squared(), p3.length_squared());
	let c2 = Vec3::new(p1.x, p2.x, p3.x);
	let c3 = Vec3::new(p1.y, p2.y, p3.y);

	let m1 = Mat3::from_cols(c2, c3, Vec3::ONE);
	let m2 = Mat3::from_cols(c1, c3, Vec3::ONE);
	let m3 = Mat3::from_cols(c1, c2, Vec3::ONE);

	Vec2::new(m2.determinant(), -m3.determinant()) * 0.5 / m1.determinant()
}
