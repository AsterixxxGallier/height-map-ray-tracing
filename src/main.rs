use crate::map::Map;
use crate::ray::{Ray2, Ray3};
use crate::tiles::{TileRegion, Tiles};
use image::{Rgb, RgbImage};
use indicatif::ProgressIterator;
use num_traits::Float;
use rayon::iter::ParallelIterator;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator};
use std::f64::consts::PI;
use std::time::Instant;
use traversal::pixel::PixelTraversal;

pub mod map;
pub mod ray;
pub mod tiles;
pub mod traversal;
pub mod transform;

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

fn main() {
    let region = TileRegion {
        x_min: 643,
        x_max: 652,
        y_min: 6858,
        y_max: 6867,
    };
    let mut tiles = Tiles::new();
    tiles.load_from_directory(region, "tiles");

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
