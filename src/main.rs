#![feature(hint_prefetch)]
#![allow(unused)]

use crate::matrix::{ArrayMatrix, Matrix};
use crate::pixel_traversal::{
    CombinedPixelTraversal,
};
use crate::ray::Ray;
use crate::ray_z::RayZ;
use image::{Rgb, RgbImage};
use num_traits::Float;
use rand::distr::Uniform;
use rand::SeedableRng;
use rand_distr::Exp1;
use rayon::iter::ParallelIterator;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator};
use std::f64::consts::PI;
use std::fmt::Debug;
use std::fs::File;
use std::time::Instant;
use tiff::decoder::{Decoder, DecodingResult};

pub mod boundary_traversal;
mod float_utils;
pub mod matrix;
pub mod pixel_traversal;
pub mod ray;
pub mod ray_z;
#[cfg(test)]
mod tests;

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

fn main() {
    let file = File::open("LHD_FXX_0648_6863_MNS_O_0M50_LAMB93_IGN69.tif").unwrap();
    let io = std::io::BufReader::new(file);
    let mut reader = Decoder::new(io).unwrap();

    let mut data = DecodingResult::F32(vec![]);

    let colortype = reader.colortype().unwrap();
    let dimensions = reader.dimensions().unwrap();
    let layout = reader.read_image_to_buffer(&mut data).unwrap();

    let DecodingResult::F32(data) = data else {
        panic!()
    };

    let x_size = 2000;
    let y_size = 2000;
    let z_size = 100;
    let x_distribution = Uniform::new(0.0, x_size as f64).unwrap();
    let y_distribution = Uniform::new(0.0, y_size as f64).unwrap();
    let z_distribution = Uniform::new(0.0, z_size as f32).unwrap();
    let height_distribution = Exp1;

    let matrix = ArrayMatrix::from_vec(x_size, y_size, data);
    matrix.save_as_image(100.0, "map.png");

    // let mut rng = SmallRng::seed_from_u64(0);
    // let mut matrix = ArrayMatrix::<f32>::random(x_size, y_size, height_distribution, &mut rng);

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
        // println!("{}/{}", start_y_index + 1, y_size * start_y_resolution);
        progress_bar.inc(1);
        let start_y = (start_y_index as f64 + 0.5) / start_y_resolution as f64;
        let mut iterator = (1..y_size * angle_resolution)
            .into_par_iter()
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
                (max_z / z_size as f32 * 255.0).min(255.0) as u8
            });
        iterator.collect_into_vec(&mut values);
        // values.clear();
        // values.extend(iterator);
        for (index, &value) in values.iter().enumerate() {
            let value = Rgb([value, value, value]);
            image.put_pixel(start_y_index as u32, index as u32, value);
        }
    }
    progress_bar.finish_and_clear();
    let elapsed = start.elapsed();
    let num_rays = image.width() * image.height();
    println!(
        "{} rays computed in {:?} ({:.2} million rays per second, {:.2} fs per ray pixel, {:.2} trillion ray pixels per second)",
        num_rays,
        elapsed,
        (num_rays as f64 / elapsed.as_secs_f64() / 1e6),
        elapsed.as_nanos() as f64 / (num_rays as usize * matrix.len()) as f64 * 1e6,
        (num_rays as f64 * matrix.len() as f64 / elapsed.as_secs_f64() / 1e12),
    );
    image.save("out.png");
}
