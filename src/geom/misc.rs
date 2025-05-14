use bevy::{
	color::{Alpha, Color},
	gizmos::gizmos::Gizmos,
	math::Vec2,
};
use petgraph::graph::UnGraph;

use crate::{geom::arc::Arc, geom::circle::Circle, util::color_hash};

pub trait DrawableWithGizmos {
	fn draw_gizmos(&self, gizmos: &mut Gizmos, color: Color);
}

pub fn show_arc_graph(graph: &UnGraph<Arc, Vec2>, gizmos: &mut Gizmos) {
	for i in graph.node_indices() {
		let arc = graph.node_weight(i).unwrap();
		arc.draw_gizmos(gizmos, color_hash(i.index()));
	}
	for i in graph.edge_indices() {
		let (j, k) = graph.edge_endpoints(i).unwrap();
		let &p = graph.edge_weight(i).unwrap();
		Circle::new(4.0, p)
			.draw_gizmos(gizmos, color_hash(j.index()).with_alpha(0.3));
		Circle::new(6.0, p)
			.draw_gizmos(gizmos, color_hash(k.index()).with_alpha(0.3));
	}
}
