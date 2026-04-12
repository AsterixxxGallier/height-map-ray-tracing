use crate::map::Map;
use crate::nodes::{read_nodes, Node};
use crate::ray::{Ray2, Ray3};
use crate::tiles::{TileCoordinates, TileRegion, Tiles};
use crate::transform::TileSpacePositionAcrossTiles;
use image::{Rgb, RgbImage};
use indicatif::ProgressIterator;
use num_traits::Float;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator};
use std::collections::HashMap;
use std::f64::consts::{FRAC_PI_2, PI};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;
use traversal::pixel::PixelTraversal;

pub mod map;
pub mod nodes;
pub mod ray;
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

pub fn max_z<T: Float>(map: &Map<f32>, ray: Ray2<T>) -> Option<f32> {
    PixelTraversal::new(ray)
        .map(|segment| map.get(segment.pixel_x as usize, segment.pixel_y as usize))
        .reduce(|a, b| a.max(b))
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
        is_line_free(tile, ray)
    })
}

pub fn max_z_across_tiles(tiles: &Tiles, ray: Ray2<f64>) -> f32 {
    tile_rays(ray)
        .map(|tile_ray| {
            let tile = tiles.tile(tile_ray.tile_coordinates).unwrap();
            max_z(tile, tile_ray.ray).unwrap()
        })
        .reduce(|a, b| a.max(b))
        .unwrap()
}

const _EIFFEL_ID: u64 = 55697;

trait _IteratorExtension: Iterator + Sized {
    fn collect_into_vec(self, vec: &mut Vec<Self::Item>) {
        vec.clear();
        vec.extend(self);
    }
}

impl<I: Iterator> _IteratorExtension for I {}

fn float_range(start: f64, end: f64, steps: usize) -> impl ExactSizeIterator<Item = f64> {
    (0..steps).map(move |index| (index as f64 + 0.5) / (steps as f64) * (end - start) + start)
}

fn _par_float_range(start: f64, end: f64, steps: usize) -> impl IndexedParallelIterator<Item = f64> {
    (0..steps)
        .into_par_iter()
        .map(move |index| (index as f64 + 0.5) / (steps as f64) * (end - start) + start)
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
    // tiles.as_debug_image(region, 200.0).save("map_big.png").unwrap();

    let mut nodes = read_nodes("nodes.csv");
    let prev_len = nodes.len();
    nodes.retain(|node| {
        let position = node.position();
        let position: TileSpacePositionAcrossTiles = position.into();
        position.x >= region.x_min as f64
            && position.x <= (region.x_max + 1) as f64
            && position.y >= region.y_min as f64
            && position.y <= (region.y_max + 1) as f64
    });
    println!("{} of {prev_len} retained", nodes.len());
    /*let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for &node in &nodes {
        let position = node.position();
        let position: TileSpacePositionAcrossTiles = position.into();
        // assert!(position.x >= 642.0, "{node:#?}\n{position:#?}");
        // assert!(position.x <= 654.0, "{node:#?}\n{position:#?}");
        // assert!(position.y >= 6857.0, "{node:#?}\n{position:#?}");
        // assert!(position.y <= 6869.0, "{node:#?}\n{position:#?}");
        x_min = x_min.min(position.x);
        x_max = x_max.max(position.x);
        y_min = y_min.min(position.y);
        y_max = y_max.max(position.y);
    }
    println!("x_min = {x_min:>10.5}");
    println!("x_max = {x_max:>10.5}");
    println!("y_min = {y_min:>10.5}");
    println!("y_max = {y_max:>10.5}");*/

    /*let mut image = tiles.as_image(region, 200.0);
    for node in nodes.iter().progress() {
        let position: PixelSpacePositionAcrossTiles = node.position().into();
        // dbg!(position);
        for i in -20..=20 {
            for j in -20..=20 {
                let x = (position.x as i32 - region.x_min * 2000).saturating_add(i) as u32;
                let y = (20_000 - 1 - (position.y as i32 - region.y_min * 2000)).saturating_add(j) as u32;
                if x >= 20_000 || y >= 20_000 {
                    continue;
                }
                if node.id == 55697 {
                    image.put_pixel(x, y, Rgb([0, 0, 255]));
                } else {
                    image.put_pixel(x, y, Rgb([0, 255, 0]));
                }
            }
        }
    }
    // image.save("map_big_nodes.png").unwrap();
    // return;

    // let ray_count = (nodes.len() * (nodes.len() - 1) / 2) as u64;
    let ray_count = (nodes.len() - 1) as u64;
    // let mut image = tiles.as_image(region, 200.0);
    // let rays = node_rays(&nodes);
    for mut ray in rays_from(&nodes, eiffel).progress_count(ray_count) {
        // if rand::random_bool(0.999) {
        //     continue;
        // }
        let is_free = is_line_free_across_tiles(&tiles, ray);
        ray.start_x *= 2000.0;
        ray.start_y *= 2000.0;
        ray.diff_x *= 2000.0;
        ray.diff_y *= 2000.0;
        for segment in PixelTraversal::new(ray.as_ray_2()) {
            let x = (segment.pixel_x - region.x_min * 2000) as u32;
            let y = (20_000 - 1 - (segment.pixel_y - region.y_min * 2000)) as u32;
            if is_free {
                image.put_pixel(x, y, Rgb([0, 255, 0]));
            } else {
                image.put_pixel(x, y, Rgb([255, 0, 0]));
            }
        }
    }
    image.save("map_big_rays.png").unwrap();
    return;*/

    /*let rays = node_rays(&nodes);
    let ray_count = (nodes.len() * (nodes.len() - 1) / 2) as u64;
    dbg!(nodes.len());
    dbg!(ray_count);
    let start = Instant::now();
    let free_count = rays
        .par_bridge()
        .progress_count(ray_count)
        .filter(|&ray| {
            // println!("ray: {ray:#?}");
            // println!("is free: {}", is_line_free_across_tiles(&tiles, ray));
            is_line_free_across_tiles(&tiles, ray)
        })
        .count();
    println!("{:.3}", free_count as f64 / ray_count as f64);
    dbg!(start.elapsed());
    return;*/

    let start_y_steps = 4000;
    let angle_steps = 4000;
    let mut image = RgbImage::new(start_y_steps as u32, angle_steps as u32);
    // let mut values = Vec::with_capacity(angle_steps);
    let start = Instant::now();
    #[derive(Copy, Clone)]
    struct RayInfo {
        start_y_index: usize,
        angle_index: usize,
        ray: Ray2<f64>,
    }
    let rays: Vec<_> = float_range(0.0, 10.0, start_y_steps)
        .enumerate()
        .flat_map(|(start_y_index, start_y)| {
            float_range(-FRAC_PI_2, FRAC_PI_2, angle_steps)
                .enumerate()
                .map(move |(angle_index, angle)| {
                    let slope = angle.tan();
                    let mut ray = Ray2 {
                        start_x: 0.0,
                        start_y,
                        diff_x: 10.0,
                        diff_y: 10.0 * slope,
                    };
                    if ray.end_y() < 0.0 {
                        let dist_y = start_y;
                        ray.diff_x = (-dist_y / slope).clamp(0.0, 10.0);
                        ray.diff_y = -dist_y;
                    } else if ray.end_y() > 10.0 {
                        let dist_y = 10.0 - start_y;
                        ray.diff_x = (dist_y / slope).clamp(0.0, 10.0);
                        ray.diff_y = dist_y;
                    }
                    ray.start_x += region.x_min as f64;
                    ray.start_y += region.y_min as f64;
                    RayInfo {
                        start_y_index,
                        angle_index,
                        ray,
                    }
                })
        })
        .collect();
    let mut tile_rays: HashMap<TileCoordinates, Vec<(Ray2<f64>, usize)>> =
        HashMap::with_capacity(region.area());
    for (ray_index, ray_info) in rays.iter().enumerate() {
        for tile_ray in self::tile_rays(ray_info.ray) {
            tile_rays
                .entry(tile_ray.tile_coordinates)
                .or_default()
                .push((tile_ray.ray, ray_index));
        }
    }
    let results = rays
        .iter()
        .map(|_| AtomicU32::default())
        .collect::<Vec<_>>();
    region.coordinates().progress_count(region.area() as u64).for_each(|tile_coordinates| {
        if let Some(tile_rays) = tile_rays.get(&tile_coordinates) {
            let tile = tiles.tile(tile_coordinates).unwrap();
            // dbg!(tile_rays.len());
            tile_rays.par_iter().for_each(|&(tile_ray, index)| {
                let result = max_z(tile, tile_ray).unwrap();
                let result = result.to_bits();
                results[index].fetch_max(result, Ordering::Relaxed);
            });
        }
    });
    for (ray_info, result) in rays.iter().zip(results).progress_count(rays.len() as u64) {
        let max_z = f32::from_bits(result.load(Ordering::Relaxed));
        let value = (max_z / 200.0 * 255.0).min(255.0) as u8;
        let value = Rgb([value, value, value]);
        image.put_pixel(
            ray_info.start_y_index as u32,
            ray_info.angle_index as u32,
            value,
        );
    }
    /*for (start_y_index, start_y) in float_range(0.0, 10.0, start_y_steps).progress().enumerate() {
        par_float_range(-FRAC_PI_2, FRAC_PI_2, angle_steps)
            .map(|angle| {
                let slope = angle.tan();
                let mut ray = Ray2 {
                    start_x: 0.0,
                    start_y,
                    diff_x: 10.0,
                    diff_y: 10.0 * slope,
                };
                if ray.end_y() < 0.0 {
                    let dist_y = start_y;
                    ray.diff_x = (-dist_y / slope).clamp(0.0, 10.0);
                    ray.diff_y = -dist_y;
                } else if ray.end_y() > 10.0 {
                    let dist_y = 10.0 - start_y;
                    ray.diff_x = (dist_y / slope).clamp(0.0, 10.0);
                    ray.diff_y = dist_y;
                }
                ray.start_x += region.x_min as f64;
                ray.start_y += region.y_min as f64;
                let max_z = max_z_across_tiles(&tiles, ray);
                (max_z / 200.0 * 255.0).min(255.0) as u8
            })
            .collect_into_vec(&mut values);
        for (index, &value) in values.iter().enumerate() {
            let value = Rgb([value, value, value]);
            image.put_pixel(start_y_index as u32, index as u32, value);
        }
    }*/

    let elapsed = start.elapsed();
    // let num_rays = image.width() * image.height();
    let num_rays = tile_rays.values().map(|rays| rays.len()).sum::<usize>();
    println!(
        "{} rays computed in {:?} ({:.2} million rays per second)",
        num_rays,
        elapsed,
        num_rays as f64 / elapsed.as_secs_f64() / 1e6,
    );
    image.save("out_big.png").unwrap();
    return;

    for coordinates in region.coordinates().progress_count(100) {
        let x_size = 2000;
        let y_size = 2000;
        let z_size = 200;

        let map = tiles.tile(coordinates).unwrap();

        let start_y_resolution = 1;
        let angle_resolution = 1;
        let mut image = RgbImage::new(
            (y_size * start_y_resolution) as u32,
            (y_size * angle_resolution) as u32 - 1,
        );
        let mut values = Vec::with_capacity(y_size * angle_resolution - 1);
        let start = Instant::now();
        let progress_bar = indicatif::ProgressBar::new((y_size * start_y_resolution) as u64);
        for start_y_index in 0..y_size * start_y_resolution {
            progress_bar.inc(1);
            let start_y = (start_y_index as f64 + 0.5) / start_y_resolution as f64;
            (1..y_size * angle_resolution)
                .into_par_iter()
                .map(|angle_index| {
                    let angle =
                        (angle_index as f64 / angle_resolution as f64 / y_size as f64 - 0.5) * PI;
                    let slope = angle.tan();
                    let mut ray = Ray2 {
                        start_x: 0.0,
                        start_y,
                        diff_x: x_size as f64,
                        diff_y: y_size as f64 * slope,
                    };
                    if ray.end_y() < 0.0 {
                        let dist_y = start_y;
                        ray.diff_x = (-dist_y / slope).clamp(0.0, y_size as f64);
                        ray.diff_y = -dist_y;
                    } else if ray.end_y() > y_size as f64 {
                        let dist_y = y_size as f64 - start_y;
                        ray.diff_x = (dist_y / slope).clamp(0.0, y_size as f64);
                        ray.diff_y = dist_y;
                    }
                    let max_z = max_z(&map, ray).unwrap();
                    (max_z / z_size as f32 * 255.0).min(255.0) as u8
                })
                .collect_into_vec(&mut values);
            for (index, &value) in values.iter().enumerate() {
                let value = Rgb([value, value, value]);
                image.put_pixel(start_y_index as u32, index as u32, value);
            }
        }
        progress_bar.finish_and_clear();

        let elapsed = start.elapsed();
        let num_rays = image.width() * image.height();
        println!(
            "{} rays computed in {:?} ({:.2} million rays per second)",
            num_rays,
            elapsed,
            num_rays as f64 / elapsed.as_secs_f64() / 1e6,
        );
        image
            .save(format!(
                "out/out_{:0>4}_{:0>4}.png",
                coordinates.x, coordinates.y
            ))
            .unwrap();
    }
}
