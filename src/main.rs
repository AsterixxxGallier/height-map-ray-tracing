use crate::curvature::curvature_drop;
use crate::map::Map;
use crate::nodes::{read_nodes, Node};
use crate::ray::{Ray2, Ray3};
use crate::tiles::{TileCoordinates, TileRegion, Tiles};
use crate::transform::TileSpacePositionAcrossTiles;
use indicatif::*;
use num_traits::Float;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;
use traversal::pixel::PixelTraversal;

pub mod curvature;
pub mod map;
pub mod nodes;
pub mod ray;
pub mod tile;
pub mod tiles;
pub mod transform;
pub mod traversal;

pub fn is_line_free<T: Float>(map: &Map<f32>, ray: Ray3<T>) -> bool {
    let mut pixel_traversal = PixelTraversal::new(ray.as_ray_2());

    if ray.diff_z >= T::zero() {
        pixel_traversal.all(|segment| {
            map.get(segment.pixel_x as usize, segment.pixel_y as usize)
                < (ray.start_z + segment.start_t * ray.diff_z)
                    .to_f32()
                    .unwrap()
        })
    } else {
        pixel_traversal.all(|segment| {
            map.get(segment.pixel_x as usize, segment.pixel_y as usize)
                < (ray.start_z + segment.end_t * ray.diff_z).to_f32().unwrap()
        })
    }
}

pub fn node_rays(nodes: &[Node]) -> impl Iterator<Item = Ray3<f64>> {
    nodes
        .iter()
        .enumerate()
        .flat_map(|(first_node_index, &first_node)| {
            nodes[..first_node_index].iter().map(move |&second_node| {
                let first_position: TileSpacePositionAcrossTiles = first_node.position().into();
                let second_position: TileSpacePositionAcrossTiles = second_node.position().into();
                Ray3 {
                    start_x: first_position.x,
                    start_y: first_position.y,
                    start_z: first_node.z,
                    diff_x: second_position.x - first_position.x,
                    diff_y: second_position.y - first_position.y,
                    diff_z: second_node.z - first_node.z,
                }
            })
        })
}

pub fn rays_from(nodes: &[Node], first_node: Node) -> impl Iterator<Item = Ray3<f64>> {
    nodes.iter().filter_map(move |&second_node| {
        if second_node.id == first_node.id {
            return None;
        }
        let first_position: TileSpacePositionAcrossTiles = first_node.position().into();
        let second_position: TileSpacePositionAcrossTiles = second_node.position().into();
        Some(Ray3 {
            start_x: first_position.x,
            start_y: first_position.y,
            start_z: first_node.z,
            diff_x: second_position.x - first_position.x,
            diff_y: second_position.y - first_position.y,
            diff_z: second_node.z - first_node.z,
        })
    })
}

#[derive(Copy, Clone)]
pub struct TileRay {
    pub tile_coordinates: TileCoordinates,
    pub start_t: f64,
    pub end_t: f64,
    // tile-relative pixel space
    pub ray: Ray2<f64>,
}

// argument in tile space
pub fn tile_rays(ray: Ray2<f64>) -> impl Iterator<Item = TileRay> {
    PixelTraversal::new(ray).map(move |tile_segment| {
        let tile_coordinates = TileCoordinates {
            x: tile_segment.pixel_x,
            y: tile_segment.pixel_y,
        };
        let sub_ray = ray.sub_ray(tile_segment.start_t, tile_segment.end_t);
        let sub_ray_start = TileSpacePositionAcrossTiles {
            x: sub_ray.start_x,
            y: sub_ray.start_y,
        };
        let sub_ray_end = TileSpacePositionAcrossTiles {
            x: sub_ray.end_x(),
            y: sub_ray.end_y(),
        };
        let sub_ray_start_position = sub_ray_start.position_in(tile_coordinates);
        let sub_ray_end_position = sub_ray_end.position_in(tile_coordinates);
        let ray_in_tile = Ray2 {
            start_x: sub_ray_start_position.x,
            start_y: sub_ray_start_position.y,
            diff_x: sub_ray_end_position.x - sub_ray_start_position.x,
            diff_y: sub_ray_end_position.y - sub_ray_start_position.y,
        };

        TileRay {
            tile_coordinates,
            start_t: tile_segment.start_t,
            end_t: tile_segment.end_t,
            ray: ray_in_tile,
        }
    })
}

pub fn is_line_free_across_tiles(tiles: &Tiles, ray: Ray3<f64>) -> bool {
    tile_rays(ray.as_ray_2()).all(|tile_ray| {
        let ray = tile_ray.ray.with_z(
            ray.start_z + tile_ray.start_t * ray.diff_z,
            ray.diff_z * (tile_ray.end_t - tile_ray.start_t),
        );
        let tile = tiles.tile(tile_ray.tile_coordinates).unwrap();
        tile.is_line_free(ray)
    })
}

pub fn tile_rays_by_tile(
    rays: impl Iterator<Item = Ray2<f64>>,
) -> HashMap<TileCoordinates, Vec<(TileRay, usize)>> {
    let mut tile_rays_by_tile: HashMap<TileCoordinates, Vec<(TileRay, usize)>> = HashMap::new();
    for (ray_index, ray) in rays.enumerate() {
        for tile_ray in tile_rays(ray) {
            tile_rays_by_tile
                .entry(tile_ray.tile_coordinates)
                .or_default()
                .push((tile_ray, ray_index));
        }
    }
    tile_rays_by_tile
}

#[allow(unreachable_code)]
fn main() {
    let region = TileRegion {
        x_min: 643,
        x_max: 652,
        y_min: 6858,
        y_max: 6867,
    };
    let mut tiles = Tiles::new();
    tiles.load_from_directory(region, "tiles");

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

    let rays = node_rays(&nodes).collect::<Vec<_>>();
    let tile_rays = tile_rays_by_tile(rays.iter().map(|ray| ray.as_ray_2()));
    let is_free = rays
        .iter()
        .map(|_| AtomicBool::new(true))
        .collect::<Vec<_>>();
    let tile_rays_checked = AtomicUsize::new(0);
    let start = Instant::now();

    for (&tile_coordinates, tile_rays) in tile_rays.iter().progress() {
        let tile = tiles.tile(tile_coordinates).unwrap();
        tile_rays.par_iter().for_each(|&(tile_ray, ray_index)| {
            if is_free[ray_index].load(Ordering::Relaxed) == false {
                // ray already intersects in other tile
                return;
            }

            tile_rays_checked.fetch_add(1, Ordering::Relaxed);

            let whole_ray = &rays[ray_index];
            // whole_ray coordinates are in tile space, where 1.0 is 1000 m
            let whole_ray_length_in_meters =
                (whole_ray.diff_x * whole_ray.diff_x + whole_ray.diff_y * whole_ray.diff_y).sqrt()
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
        });
    }

    let duration = start.elapsed();
    let free_count = is_free
        .iter()
        .filter(|free| free.load(Ordering::Relaxed))
        .count();
    let total_count = is_free.len();
    let whole_ray_count = rays.len();
    let tile_ray_count = tile_rays_checked.load(Ordering::Relaxed);
    println!(
        "{:.2}% free ({free_count} of {total_count})",
        free_count as f64 / total_count as f64 * 100.0
    );
    println!(
        "{:.2} tile rays checked per ray",
        tile_ray_count as f64 / whole_ray_count as f64
    );
    println!(
        "{:.2} million tile rays checked per second",
        tile_ray_count as f64 / duration.as_secs_f64() / 1e6
    );
    println!("took {:?}", duration);
}
