use std::{
	f32::consts::{FRAC_PI_2, PI},
	iter::Sum,
	ops::Add,
};

use bevy::{
	color::Color,
	gizmos::gizmos::Gizmos,
	math::{Mat2, Vec2},
	platform::collections::{HashMap, HashSet},
};
use derive_more::{Deref, DerefMut};
use itertools::Itertools;
use petgraph::{
	Direction::Outgoing,
	graph::{EdgeIndex, EdgeReference, Graph, NodeIndex},
	visit::EdgeRef,
};

use crate::{
	constants::GENERAL_EPSILON,
	geom::{arc::Arc, circle::Circle, misc::DrawableWithGizmos},
	math::{diff_ccw, diff_cw},
	util::color_hash,
};

#[derive(Clone, Default, Deref, DerefMut)]
pub struct ArcGraph(pub Graph<Arc, Vec2>);

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
			let j = node_index_map[&j];
			let (a, b) = (self.node_weight(i).unwrap(), self.node_weight(j).unwrap());
			let d = Mat2::from_cols(p - a.center, p - b.center).determinant();
			let (i_, j_) = if a.span * b.span * d > 0.0 { (j, i) } else { (i, j) };
			self.add_edge(i_, j_, p);
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
	pub fn remove_edges(&mut self, ids: HashSet<EdgeIndex>) {
		let mut edges_to_keep = vec![];
		let ids_to_keep: HashSet<EdgeIndex> =
			&self.edge_indices().collect::<HashSet<EdgeIndex>>() - &ids;
		ids_to_keep.iter().unique().for_each(|&i| {
			let e = self.edge_references().find(|e| e.id() == i).unwrap();
			edges_to_keep.push((e.source(), e.target(), *e.weight()));
		});
		self.clear_edges();
		self.extend_with_edges(edges_to_keep);
	}

	pub fn minkowski(arcs: Vec<Arc>, radius: f32) -> Self {
		let m_arcs = arcs.iter().map(|&a| Self::minkowski_arc(a, radius));
		let mut sum: ArcGraph = m_arcs.sum();
		let mut edge_ids_to_remove = HashSet::new();
		for eref in sum.edge_references() {
			let (i, &p) = (eref.id(), eref.weight());
			for &arc in arcs.iter() {
				if arc.distance_to_point(p) - radius < -GENERAL_EPSILON {
					edge_ids_to_remove.insert(i);
					continue;
				}
			}
		}
		sum.remove_edges(edge_ids_to_remove);
		let mut g = ArcGraph::default();
		for eref in sum.edge_references() {
			let (target_id, &p) = (eref.target(), eref.weight());
			let &target_arc = sum.node_weight(target_id).unwrap();
			let angle_diff_func =
				if target_arc.span < 0.0 { diff_cw } else { diff_ccw };
			let c = target_arc.center;
			let current_angle = (p - c).to_angle();
			let edge_to_order = |e: EdgeReference<Vec2>| {
				let e_angle = (e.weight() - c).to_angle();
				angle_diff_func(current_angle, e_angle)
			};
			let mut next_outgoing: Vec<(EdgeReference<Vec2>, f32)> = sum
				.edges_directed(target_id, Outgoing)
				.filter(|e| e.id() != eref.id())
				.map(|e| (e, edge_to_order(e)))
				.collect();
			next_outgoing.sort_by(|(_, x), (_, y)| x.total_cmp(y));
			if let Some((_next, x)) = next_outgoing.first() {
				let arc_init_func = if target_arc.span < 0.0 {
					Arc::from_angles_cw
				} else {
					Arc::from_angles_ccw
				};
				let arc = arc_init_func(
					current_angle,
					current_angle + x * target_arc.span.signum(),
					target_arc.radius,
					target_arc.center,
				);
				g.add_node(arc);
			}
		}
		g
	}

	pub fn minkowski_arc(arc: Arc, radius: f32) -> Self {
		let mut g = Graph::<Arc, Vec2>::new();
		let idx1 = g.add_node(arc.with_radius(arc.radius + radius));
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
			let idx2 = g.add_node(end_point_arc);
			let idx3 =
				g.add_node(arc.with_radius(arc.radius - radius).with_span(-arc.span));
			let idx4 = g.add_node(start_point_arc);
			g.add_edge(idx1, idx2, end_point_arc.start_point());
			g.add_edge(idx2, idx3, end_point_arc.end_point());
			g.add_edge(idx3, idx4, start_point_arc.start_point());
			g.add_edge(idx4, idx1, start_point_arc.end_point());
		} else {
			if let Some(&intersection) = Circle::new(radius, arc.start_point())
				.intersect(Circle::new(radius, arc.end_point()))
				.get((0.5 * (arc.span.signum() + 1.0)) as usize)
			{
				let f = if arc.span < 0.0 {
					Arc::from_angles_cw
				} else {
					Arc::from_angles_ccw
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
				let idx2 = g.add_node(end_point_arc);
				let idx3 = g.add_node(start_point_arc);
				g.add_edge(idx1, idx2, end_point_arc.start_point());
				g.add_edge(idx2, idx3, end_point_arc.end_point());
				g.add_edge(idx3, idx1, start_point_arc.end_point());
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
			a.intersect(b).iter().for_each(|&p| res.push((i, j, p)));
		}
		res
	}
}
