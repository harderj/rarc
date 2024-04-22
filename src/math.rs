
use bevy::math::{Mat2, Mat3, Vec2, Vec3};

pub fn second_deg_eq(a: f32, b: f32, c: f32) -> Vec<f32> {
    let d = b.powi(2) - 4.0 * a * c;
    if d < 0.0 { Vec::new() }
    else if d == 0.0 { Vec::from([-b / (2.0 * a)]) }
    else {
        let sqrt_d = d.sqrt();
        let v: Vec2 = (Vec2::new(-sqrt_d, sqrt_d) - b) / (2.0 * a);
        Vec::from([v.min_element(), v.max_element()])
    }
}

pub fn circle_center_from_3_points(p1: Vec2, p2: Vec2, p3: Vec2) -> Vec2 {
	let c1 = Vec3::new(p1.length_squared(), p2.length_squared(), p3.length_squared());
	let c2 = Vec3::new(p1.x, p2.x, p3.x);
	let c3 = Vec3::new(p1.y, p2.y, p3.y);

	let m1 = Mat3::from_cols(c2, c3, Vec3::ONE);
	let m2 = Mat3::from_cols(c1, c3, Vec3::ONE);
	let m3 = Mat3::from_cols(c1, c2, Vec3::ONE);

	Vec2::new(m2.determinant(), -m3.determinant()) * 0.5 / m1.determinant()
}

pub struct Circle {
    pub center: Vec2,
    pub radius: f32
}

#[derive(Default)]
pub struct Collision {
    pub point: Vec2,
    pub time: f32
}

fn three_circle_collision_0(a: Circle, b: Circle) -> Option<Collision> {
    let m = Mat2::from_cols(a.center, b.center).transpose();
    let alpha = 1.0 / (2.0 * m.determinant());
    let beta_a = a.center.length_squared() - a.radius.powi(2);
    let beta_b = b.center.length_squared() - b.radius.powi(2);
    let gamma_a = -2.0 * a.radius;
    let gamma_b = -2.0 * b.radius;
    let delta_x = alpha * (b.center.y * gamma_a - a.center.y * gamma_b);
    let delta_y = alpha * (-b.center.x * gamma_a + a.center.x * gamma_b);
    let epsilon_x = alpha * (b.center.y * beta_a - a.center.y * beta_b);
    let epsilon_y = alpha * (-b.center.x * beta_a + a.center.x * beta_b);
    let eq_a = delta_x.powi(2) + delta_y.powi(2) - 1.0;
    let eq_b = 2.0 * (delta_x * epsilon_x + delta_y * epsilon_y);
    let eq_c = epsilon_x.powi(2) + epsilon_y.powi(2);
    let pot_t = second_deg_eq(eq_a, eq_b, eq_c);
    let t = match pot_t.len() {
        0 => None,
        1 => {
            let t = *pot_t.first().unwrap();
            if t > 0.0 { Some(t) } else { None }
        },
        2 => {
            let mut t: f32 = *pot_t.first().unwrap();
            if t < 0.0 { t = *pot_t.get(1).unwrap(); }
            if t > 0.0 { Some(t) } else { None }
        },
        _ => panic!("Not possible.")
    };
    if t.is_none() { None }
    else {
        Some(Collision { time: t.unwrap(), point: Vec2::new(
            delta_x * t.unwrap() + epsilon_x,
            delta_y * t.unwrap() + epsilon_x
        ) })
    }
}

