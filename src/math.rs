use bevy::math::{Mat3, Vec2, Vec3};

pub fn circle_center_from_3_points(p1: Vec2, p2: Vec2, p3: Vec2) -> Vec2 {
	let c1 = Vec3::new(p1.length_squared(), p2.length_squared(), p3.length_squared());
	let c2 = Vec3::new(p1.x, p2.x, p3.x);
	let c3 = Vec3::new(p1.y, p2.y, p3.y);

	let m1 = Mat3::from_cols(c2, c3, Vec3::ONE);
	let m2 = Mat3::from_cols(c1, c3, Vec3::ONE);
	let m3 = Mat3::from_cols(c1, c2, Vec3::ONE);

    // println!("p1: {}", p1);
    // println!("p2: {}", p2);
    // println!("p3: {}", p3);
    // println!("c1: {}", c1);
    // println!("c2: {}", c2);
    // println!("c3: {}", c3);
    // println!("m1: {}", m1);
    // println!("m2: {}", m2);
    // println!("m3: {}", m3);

    // panic!("lala");

	Vec2::new(m2.determinant(), -m3.determinant()) * 0.5 / m1.determinant()
}
