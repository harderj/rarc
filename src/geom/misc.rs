use bevy::{color::Color, gizmos::gizmos::Gizmos};

pub trait DrawableWithGizmos {
	fn draw_gizmos(&self, gizmos: &mut Gizmos, color: Color);
}
