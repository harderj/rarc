use bevy::{color::Color, gizmos::gizmos::Gizmos};

#[derive(Default)]
pub struct DrawGizmosOptions {
	pub color: Option<Color>,
	pub directions_indicators: bool,
}

impl DrawGizmosOptions {
	pub fn from_color(color: Color) -> Self {
		DrawGizmosOptions { color: Some(color), ..Default::default() }
	}
}

pub trait DrawableWithGizmos {
	fn draw_gizmos(&self, gizmos: &mut Gizmos, options: &DrawGizmosOptions);
}
