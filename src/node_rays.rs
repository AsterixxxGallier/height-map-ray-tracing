use rayon::prelude::*;
use crate::nodes::Node;
use crate::ray::Ray3;
use crate::transform::TileSpacePositionAcrossTiles;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct NodeRayId {
    pub first_node_index: usize,
    pub second_node_index: usize,
}

pub struct NodeRays<'a> {
    nodes: &'a [Node],
    max_ray_length_km_squared: f64,
}

fn ray(first_node: Node, second_node: Node) -> Ray3<f64> {
    let first_position: TileSpacePositionAcrossTiles = first_node.position().into();
    let second_position: TileSpacePositionAcrossTiles =
        second_node.position().into();

    Ray3 {
        start_x: first_position.x,
        start_y: first_position.y,
        start_z: first_node.z,
        diff_x: second_position.x - first_position.x,
        diff_y: second_position.y - first_position.y,
        diff_z: second_node.z - first_node.z,
    }
}

impl<'a> NodeRays<'a> {
    pub fn new(nodes: &'a [Node], max_ray_length: f64) -> Self {
        let max_ray_length_km = max_ray_length / 1000.0;
        let max_ray_length_km_squared = max_ray_length_km * max_ray_length_km;
        Self {
            nodes,
            max_ray_length_km_squared,
        }
    }

    pub fn ray(&self, id: NodeRayId) -> Ray3<f64> {
        let first_node = self.nodes[id.first_node_index];
        let second_node = self.nodes[id.second_node_index];

        ray(first_node, second_node)
    }

    pub fn count(&self) -> usize {
        self.iter().count()
    }

    pub fn iter(&self) -> impl Iterator<Item=(NodeRayId, Ray3<f64>)> {
        self.nodes
            .iter()
            .enumerate()
            .flat_map(move |(first_node_index, &first_node)| {
                self.nodes[..first_node_index]
                    .iter()
                    .enumerate()
                    .filter_map(move |(second_node_index, &second_node)| {
                        if !first_node.active && !second_node.active {
                            return None;
                        }

                        let ray = ray(first_node, second_node);

                        if ray.as_ray_2().length_squared() > self.max_ray_length_km_squared {
                            return None;
                        }

                        let id = NodeRayId {
                            first_node_index,
                            second_node_index,
                        };

                        Some((id, ray))
                    })
            })
    }

    pub fn par_iter(&self) -> impl ParallelIterator<Item=(NodeRayId, Ray3<f64>)> {
        self.nodes
            .par_iter()
            .enumerate()
            .flat_map(move |(first_node_index, &first_node)| {
                self.nodes[..first_node_index]
                    .par_iter()
                    .enumerate()
                    .filter_map(move |(second_node_index, &second_node)| {
                        if !first_node.active && !second_node.active {
                            return None;
                        }

                        let ray = ray(first_node, second_node);

                        if ray.as_ray_2().length_squared() > self.max_ray_length_km_squared {
                            return None;
                        }

                        let id = NodeRayId {
                            first_node_index,
                            second_node_index,
                        };

                        Some((id, ray))
                    })
            })
    }
}
