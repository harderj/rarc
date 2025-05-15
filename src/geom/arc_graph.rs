use std::{
	f32::consts::{FRAC_PI_2, PI},
	iter::Sum,
	ops::Add,
};

use bevy::{
	color::Color, gizmos::gizmos::Gizmos, math::Vec2,
	platform::collections::HashMap,
};
use derive_more::{Deref, DerefMut};
use itertools::Itertools;
use petgraph::{
	graph::{NodeIndex, UnGraph},
	visit::EdgeRef,
};

use crate::{
	constants::GENERAL_EPSILON,
	geom::{arc::Arc, circle::Circle, misc::DrawableWithGizmos},
	util::color_hash,
};

#[derive(Clone, Default, Deref, DerefMut)]
pub struct ArcGraph(pub UnGraph<Arc, Vec2>);

impl Add for ArcGraph {
	type Output = ArcGraph;

	fn add(mut self, rhs: Self) -> Self::Output {
		let intersections = self.intersect(&rhs);
		let mut node_index_map = HashMap::new();
		for i in rhs.node_indices() {
			let &arc = rhs.node_weight(i).unwrap();
			let ni = self.add_node(arc);
			node_index_map.insert(i, ni);
		}
		for eref in rhs.edge_references() {
			let (i, j, p) = (eref.source(), eref.target(), eref.weight());
			self.add_edge(node_index_map[&i], node_index_map[&j], *p);
		}
		for (i, j, p) in intersections {
			self.add_edge(i, node_index_map[&j], p);
		}
		self
	}
}

impl Sum for ArcGraph {
	fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
		iter.fold(ArcGraph::default(), |a, b| a + b)
	}
}

impl DrawableWithGizmos for ArcGraph {
	fn draw_gizmos(&self, gizmos: &mut Gizmos, color: Option<Color>) {
		let color_f = |i: NodeIndex| Some(color.unwrap_or(color_hash(i.index())));
		for i in self.node_indices() {
			let arc = self.node_weight(i).unwrap();
			arc.draw_gizmos(gizmos, color_f(i));
		}
		for eref in self.edge_references() {
			let (i, j, &p) = (eref.source(), eref.target(), eref.weight());
			Circle::new(3.0, p).draw_gizmos(gizmos, color_f(i));
			Circle::new(6.0, p).draw_gizmos(gizmos, color_f(j));
		}
	}
}

impl ArcGraph {
	pub fn minkowski(arcs: Vec<Arc>, radius: f32) -> Self {
		let m_arcs = arcs.iter().map(|&a| Self::minkowski_arc(a, radius));
		let mut sum: ArcGraph = m_arcs.sum();
		let mut edge_ids_to_remove = vec![];
		for eref in sum.edge_references() {
			let (i, &p) = (eref.id(), eref.weight());
			for arc in &arcs {
				if arc.distance_to_point(p) - radius < -GENERAL_EPSILON {
					edge_ids_to_remove.push(i)
				}
			}
		}
		edge_ids_to_remove.iter().for_each(|&i| {
			sum.remove_edge(i);
		});
		sum
		// todo: pick all edges and move one along direction
	}

	pub fn minkowski_arc(arc: Arc, radius: f32) -> Self {
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
				g.add_edge(_idx3, _idx1, start_point_arc.end_point());
			}
		}
		ArcGraph(g)
	}

	pub fn intersect(
		&self,
		other: &ArcGraph,
	) -> Vec<(NodeIndex, NodeIndex, Vec2)> {
		let mut res = vec![];
		for (i, j) in self.node_indices().cartesian_product(other.node_indices()) {
			let (&a, &b) = (
				self.node_weight(i).unwrap(), //
				other.node_weight(j).unwrap(),
			);
			let ps = a.intersect(b);
			ps.into_iter().for_each(|p| res.push((i, j, p)));
		}
		res
	}
}
