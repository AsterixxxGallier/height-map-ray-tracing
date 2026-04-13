use crate::cli::Args;
use crate::curvature::curvature_drop;
use crate::node_rays::NodeRays;
use crate::nodes::read_nodes;
use crate::tile_rays::par_tile_rays_for_tile;
use crate::tiles::{TileRegion, download_and_load_tile};
use crate::transform::TileSpacePositionAcrossTiles;
use clap::Parser;
use indicatif::*;
use rayon::iter::ParallelIterator;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

pub mod cli;
pub mod curvature;
pub mod intersection;
pub mod map;
pub mod node_rays;
pub mod nodes;
pub mod ray;
pub mod tile;
pub mod tile_rays;
pub mod tiles;
pub mod transform;
pub mod traversal;

#[allow(unreachable_code)]
fn main() {
    let Args { max_link_length } = Args::parse();

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

    let node_rays = NodeRays::new(&nodes, max_link_length);

    let mut is_free = (0..nodes.len() * nodes.len())
        .map(|_| AtomicBool::new(true))
        .collect::<Vec<_>>();
    for (id, _) in node_rays.iter() {
        *is_free[id.first_node_index * nodes.len() + id.second_node_index].get_mut() = true;
    }

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
            let tile_rays = par_tile_rays_for_tile(
                tile_coordinates,
                node_rays.par_iter().map(|(id, ray)| (id, ray.as_ray_2())),
            );

            // `a` is the number of tile rays in the iterator,
            // `b` is the number of tile rays we actually had to check.
            // This counting method is a bit clunky, but it's much more performant than atomics.
            let (a, b) = tile_rays
                .map(|(tile_ray, ray_id)| {
                    let is_free_index =
                        ray_id.first_node_index * nodes.len() + ray_id.second_node_index;

                    if is_free[is_free_index].load(Ordering::Relaxed) == false {
                        // ray already intersects in other tile
                        return (1, 0);
                    }

                    let whole_ray = node_rays.ray(ray_id);
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
                        is_free[is_free_index].store(false, Ordering::Relaxed);
                    }

                    (1, 1)
                })
                .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
            (a, b)
        })
        .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1));

    let duration = start.elapsed();
    let free_count = is_free
        .iter()
        .filter(|free| free.load(Ordering::Relaxed))
        .count();
    let total_count = node_rays.count();
    println!(
        "{:.2}% free ({free_count} of {total_count})",
        free_count as f64 / total_count as f64 * 100.0
    );
    println!(
        "{:.2} tile rays checked per ray",
        checked_tile_ray_count as f64 / total_count as f64
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
