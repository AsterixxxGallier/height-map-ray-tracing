#![allow(unused)]

use crate::combined_pixel_traversal::CombinedPixelTraversal;
use crate::matrix::{ArrayMatrix, Matrix};
use crate::ray::Ray;
use crate::ray_z::RayZ;
use crate::separate_pixel_traversal::{XPixelTraversal, YPixelTraversal};
use image::{Rgb, RgbImage};
use num_traits::Float;
use rand::distr::Uniform;
use rand::prelude::SmallRng;
use rand::SeedableRng;
use rand_distr::Exp1;
use std::f64::consts::PI;
use std::fmt::Debug;
use std::time::Instant;
use crate::separate_traversal::{XBoundaryTraversal, YBoundaryTraversal};

pub mod bounds;
pub mod combined_pixel_traversal;
pub mod combined_traversal;
pub mod in_bounds_combined_traversal;
pub mod matrix;
pub mod ray;
pub mod ray_z;
pub mod separate_pixel_traversal;
pub mod separate_traversal;
#[cfg(test)]
mod tests;
pub mod thin_combined_traversal;

pub fn is_line_free<M: Matrix<Item = f32>, T: Float>(matrix: &M, ray_z: RayZ<T>) -> bool {
    let ray = ray_z.as_ray();
    let mut pixel_traversal = CombinedPixelTraversal::new(ray);

    if ray_z.diff_z >= T::zero() {
        pixel_traversal.all(|segment| {
            matrix.get(segment.pixel_x as usize, segment.pixel_y as usize)
                < (ray_z.start_z + segment.start_t * ray_z.diff_z)
                    .to_f32()
                    .unwrap()
        })
    } else {
        pixel_traversal.all(|segment| {
            matrix.get(segment.pixel_x as usize, segment.pixel_y as usize)
                < (ray_z.start_z + segment.end_t * ray_z.diff_z)
                    .to_f32()
                    .unwrap()
        })
    }
}

pub fn max_z<M: Matrix<Item = f32>, T: Float>(matrix: &M, ray: Ray<T>) -> Option<f32> {
    let mut pixel_traversal = CombinedPixelTraversal::new(ray);

    pixel_traversal
        .map(|segment| matrix.get(segment.pixel_x as usize, segment.pixel_y as usize))
        .reduce(|a, b| a.max(b))
}

// does not work
pub fn max_z_separate<M: Matrix<Item = f32>>(matrix: &M, ray: Ray<f64>) -> Option<f32> {
    let mut x_traversal = XBoundaryTraversal::new(ray);
    let mut y_traversal = YBoundaryTraversal::new(ray);

    x_traversal
        .into_iter()
        .flatten()
        .map(|crossing| matrix.get(crossing.next_x_index as usize, crossing.y_index as usize))
        .chain(
            y_traversal
                .into_iter()
                .flatten()
                .map(|crossing| matrix.get(crossing.x_index as usize, crossing.next_y_index as usize)),
        )
        .reduce(|a, b| a.max(b))
}

fn main() {
    let x_size = 2048;
    let y_size = 2048;
    let z_size = 15;
    let x_distribution = Uniform::new(0.0, x_size as f64).unwrap();
    let y_distribution = Uniform::new(0.0, y_size as f64).unwrap();
    let z_distribution = Uniform::new(0.0, z_size as f32).unwrap();
    let height_distribution = Exp1;

    let mut rng = SmallRng::seed_from_u64(0);
    let mut matrix = ArrayMatrix::<f32>::random(x_size, y_size, height_distribution, &mut rng);

    let start = Instant::now();
    let start_y_resolution = 1;
    let angle_resolution = 1;
    let mut image = RgbImage::new(
        (y_size * start_y_resolution) as u32,
        (y_size * angle_resolution) as u32 - 1,
    );
    for start_y_index in 0..y_size * start_y_resolution {
        println!("{}/{}", start_y_index + 1, y_size * start_y_resolution);
        let start_y = (start_y_index as f64 + 0.5) / start_y_resolution as f64;
        let values: Vec<u8> = (1..y_size * angle_resolution)
            // .into_par_iter()
            .map(|angle_index| {
                let angle =
                    (angle_index as f64 / angle_resolution as f64 / y_size as f64 - 0.5) * PI;
                let slope = angle.tan();
                let mut ray = Ray {
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
                let mut max_z = max_z(&matrix, ray).unwrap();
                (max_z / 15.0 * 255.0) as u8
            })
            .collect();
        for (index, value) in values.into_iter().enumerate() {
            let value = Rgb([value, value, value]);
            image.put_pixel(start_y_index as u32, index as u32, value);
        }
    }
    let elapsed = start.elapsed();
    let num_rays = image.width() * image.height();
    println!(
        "{} rays computed in {:?} ({:.2} fs per ray pixel, {:.2} trillion ray pixels per second)",
        num_rays,
        elapsed,
        elapsed.as_nanos() as f64 / (num_rays as usize * matrix.len()) as f64 * 1e6,
        (num_rays as f64 * matrix.len() as f64 / elapsed.as_secs_f64() / 1e12),
    );
    image.save("out.png");
}
