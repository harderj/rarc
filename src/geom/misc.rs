use crate::math::FloatVec2;

#[derive(Debug)]
pub enum Collision {
	Size(f32),
	Opposite(FloatVec2, usize, usize),
	Neighbors(FloatVec2, usize),
	RadiusZero(FloatVec2, usize),
}

impl Collision {
	pub fn offset(&self) -> f32 {
		match self {
			Collision::Size(t) => *t,
			Collision::Opposite(FloatVec2(t, _), _, _) => *t,
			Collision::Neighbors(FloatVec2(t, _), _) => *t,
			Collision::RadiusZero(FloatVec2(t, _), _) => *t,
		}
	}
}
