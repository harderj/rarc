use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{color::Color, gizmos::gizmos::Gizmos, math::Vec2};
use derive_more::Deref;
use petgraph::graph::{NodeIndex, UnGraph};

use crate::{
	geom::{arc::Arc, circle::Circle, misc::DrawableWithGizmos},
	util::color_hash,
};

#[derive(Clone, Deref)]
pub struct ArcGraph(pub UnGraph<Arc, Vec2>);

impl DrawableWithGizmos for ArcGraph {
	fn draw_gizmos(&self, gizmos: &mut Gizmos, color: Option<Color>) {
		for i in self.node_indices() {
			let arc = self.node_weight(i).unwrap();
			let color = color.unwrap_or(color_hash(i.index()));
			arc.draw_gizmos(gizmos, Some(color));
		}
		for i in self.edge_indices() {
			let color = color.unwrap_or(color_hash(i.index()));
			let &p = self.edge_weight(i).unwrap();
			Circle::new(4.0, p).draw_gizmos(gizmos, Some(color));
			Circle::new(6.0, p).draw_gizmos(gizmos, Some(color));
		}
	}
}

impl ArcGraph {
	pub fn intersect(_other: &ArcGraph) -> Vec<(NodeIndex, NodeIndex, Vec2)> {
		todo!()
	}

	pub fn minkowski_arc(arc: Arc, radius: f32) -> Self {
		// consider to make this cleaner by changing circles into arcs
		let mut g = UnGraph::<Arc, Vec2>::new_undirected();
		let _idx1 = g.add_node(arc.with_radius(arc.radius + radius));
		if radius.abs() < arc.radius.abs() {
			let end_point_arc = Arc {
				radius,
				center: arc.end_point(),
				mid: arc.end_angle() + FRAC_PI_2 * arc.span.signum(),
				span: PI * arc.span.signum(),
			};
			let start_point_arc = Arc {
				radius,
				center: arc.start_point(),
				mid: arc.start_angle() - FRAC_PI_2 * arc.span.signum(),
				span: PI * arc.span.signum(),
			};
			let _idx2 = g.add_node(end_point_arc);
			let _idx3 =
				g.add_node(arc.with_radius(arc.radius - radius).with_span(-arc.span));
			let _idx4 = g.add_node(start_point_arc);
			g.add_edge(_idx1, _idx2, end_point_arc.start_point());
			g.add_edge(_idx2, _idx3, end_point_arc.end_point());
			g.add_edge(_idx3, _idx4, start_point_arc.start_point());
			g.add_edge(_idx4, _idx1, start_point_arc.end_point());
		} else {
			if let Some(&intersection) = Circle::new(radius, arc.start_point())
				.intersect(Circle::new(radius, arc.end_point()))
				.get((0.5 * (arc.span.signum() + 1.0)) as usize)
			{
				let f = if arc.span < 0.0 {
					Arc::from_angles_clockwise
				} else {
					Arc::from_angles_counterclockwise
				};
				let end_point_arc = f(
					arc.end_angle(),
					(intersection - arc.end_point()).to_angle(),
					radius,
					arc.end_point(),
				);
				let start_point_arc = f(
					(intersection - arc.start_point()).to_angle(),
					arc.start_angle(),
					radius,
					arc.start_point(),
				);
				let _idx2 = g.add_node(end_point_arc);
				let _idx3 = g.add_node(start_point_arc);
				g.add_edge(_idx1, _idx2, end_point_arc.start_point());
				g.add_edge(_idx2, _idx3, end_point_arc.end_point());
				g.add_edge(_idx3, _idx2, start_point_arc.end_point());
			}
		}
		ArcGraph(g)
	}
}
