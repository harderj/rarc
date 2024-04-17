use std::f32::consts::PI;

use crate::math::angle_within;

#[test]
fn test_angle_within() {
	assert!(angle_within(PI, 0.0, 1.99 * PI));
	assert!(angle_within(PI, -1.0, 1.1 * PI));
	assert!(angle_within(0.0, -1.0, 2.0));
	assert!(angle_within(0.0, 2.0 * PI - 1.0, 2.0));
	assert!(angle_within(0.0, 2.0, 2.0));
	assert!(angle_within(1.0, 0.0, 2.0));
}
