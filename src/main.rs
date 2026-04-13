use crate::curvature::curvature_drop;
use crate::nodes::{read_nodes, Node};
use crate::ray::Ray3;
use crate::tile_rays::par_tile_rays_for_tile;
use crate::tiles::{download_and_load_tile, TileRegion};
use crate::transform::TileSpacePositionAcrossTiles;
use indicatif::*;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use clap::Parser;
use crate::cli::Args;

pub mod cli;
pub mod curvature;
pub mod intersection;
pub mod map;
pub mod nodes;
pub mod ray;
pub mod tile;
pub mod tile_rays;
pub mod tiles;
pub mod transform;
pub mod traversal;

pub fn node_rays(nodes: &[Node], max_length_km: f64) -> impl Iterator<Item = Ray3<f64>> {
    let max_length_squared = max_length_km * max_length_km;
    nodes
        .iter()
        .enumerate()
        .flat_map(move |(first_node_index, &first_node)| {
            nodes[..first_node_index]
                .iter()
                .filter_map(move |&second_node| {
                    if !first_node.active && !second_node.active {
                        return None;
                    }

                    let first_position: TileSpacePositionAcrossTiles = first_node.position().into();
                    let second_position: TileSpacePositionAcrossTiles =
                        second_node.position().into();

                    let distance_squared = (first_position.x - second_position.x)
                        * (first_position.x - second_position.x)
                        + (first_position.y - second_position.y)
                            * (first_position.y - second_position.y);

                    if distance_squared > max_length_squared {
                        return None;
                    }

                    Some(Ray3 {
                        start_x: first_position.x,
                        start_y: first_position.y,
                        start_z: first_node.z,
                        diff_x: second_position.x - first_position.x,
                        diff_y: second_position.y - first_position.y,
                        diff_z: second_node.z - first_node.z,
                    })
                })
        })
}

#[allow(unreachable_code)]
fn main() {
    let Args { max_link_length } = Args::parse();
    let max_link_length_km = max_link_length / 1000.0;

    let region = TileRegion {
        x_min: 643,
        x_max: 652,
        y_min: 6858,
        y_max: 6867,
    };

    let mut nodes = read_nodes("nodes.csv");

    // filter out out-of-bounds nodes
    nodes.retain(|node| {
        let position = node.position();
        let position: TileSpacePositionAcrossTiles = position.into();
        position.x >= region.x_min as f64
            && position.x <= (region.x_max + 1) as f64
            && position.y >= region.y_min as f64
            && position.y <= (region.y_max + 1) as f64
    });

    let start = Instant::now();
    let rays = node_rays(&nodes, max_link_length_km).collect::<Vec<_>>();
    let rays: Vec<_> = (0..10).flat_map(|_| rays.iter().copied()).collect();
    println!("collected rays in {:?}", start.elapsed());

    let is_free = rays
        .iter()
        .map(|_| AtomicBool::new(true))
        .collect::<Vec<_>>();
    let start = Instant::now();

    let (total_tile_ray_count, checked_tile_ray_count) = region
        .par_coordinates()
        .progress_count(region.area() as u64)
        .map(|tile_coordinates| {
            (
                tile_coordinates,
                download_and_load_tile("tiles", tile_coordinates),
            )
        })
        .map(|(tile_coordinates, tile)| {
            let tile_rays =
                par_tile_rays_for_tile(tile_coordinates, rays.par_iter().map(|ray| ray.as_ray_2()));

            tile_rays
                .map(|(tile_ray, ray_index)| {
                    if is_free[ray_index].load(Ordering::Relaxed) == false {
                        // ray already intersects in other tile
                        // `1` is for counting the tile ray
                        // `0` indicates that this tile ray is not being checked
                        return (1, 0);
                    }

                    let whole_ray = &rays[ray_index];
                    // whole_ray coordinates are in tile space, where 1.0 is 1000 m
                    let whole_ray_length_in_meters = (whole_ray.diff_x * whole_ray.diff_x
                        + whole_ray.diff_y * whole_ray.diff_y)
                        .sqrt()
                        * 1_000.0;
                    let mut start_z = whole_ray.start_z + whole_ray.diff_z * tile_ray.start_t;
                    let mut end_z = whole_ray.start_z + whole_ray.diff_z * tile_ray.end_t;
                    start_z -= curvature_drop(tile_ray.start_t, whole_ray_length_in_meters);
                    end_z -= curvature_drop(tile_ray.end_t, whole_ray_length_in_meters);
                    let ray = tile_ray.ray.with_z(start_z, end_z - start_z);
                    let free = tile.is_line_free(ray);
                    if !free {
                        is_free[ray_index].store(false, Ordering::Relaxed);
                    }

                    // `1` is for counting the tile ray
                    // `1` indicates that this tile ray has been checked
                    (1, 1)
                })
                .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
        })
        .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1));

    let duration = start.elapsed();
    let free_count = is_free
        .iter()
        .filter(|free| free.load(Ordering::Relaxed))
        .count();
    let total_count = is_free.len();
    let whole_ray_count = rays.len();
    println!(
        "{:.2}% free ({free_count} of {total_count})",
        free_count as f64 / total_count as f64 * 100.0
    );
    println!(
        "{:.2} tile rays checked per ray",
        checked_tile_ray_count as f64 / whole_ray_count as f64
    );
    println!(
        "{:.2} million tile rays checked per second",
        checked_tile_ray_count as f64 / duration.as_secs_f64() / 1e6
    );
    println!(
        "{:.2}% of tile rays checked ({checked_tile_ray_count} of {total_tile_ray_count})",
        checked_tile_ray_count as f64 / total_tile_ray_count as f64 * 100.0
    );
    println!("took {:?}", duration);
}
