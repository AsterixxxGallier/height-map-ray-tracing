#![feature(hint_prefetch)]
#![allow(unused)]

use std::arch::asm;
use crate::combined_pixel_traversal::{CombinedPixelTraversal, ThinCombinedPixelTraversal, ThinCombinedPixelTraversalWithoutT};
use crate::matrix::{ArrayMatrix, Matrix};
use crate::ray::Ray;
use crate::ray_z::RayZ;
use crate::separate_traversal::{XBoundaryTraversal, YBoundaryTraversal};
use image::{Rgb, RgbImage};
use indicatif::ProgressStyle;
use num_traits::Float;
use rand::distr::Uniform;
use rand::SeedableRng;
use rand_distr::Exp1;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator};
use rayon::iter::ParallelIterator;
use std::f64::consts::PI;
use std::fmt::Debug;
use std::fs::File;
use std::hint::{black_box, prefetch_read, Locality};
use std::time::Instant;
use loop_code::repeat;
use tiff::decoder::{Decoder, DecodingResult};
use crate::float_utils::{integers_between, CombinedBoundaryTraversalVariables};
use crate::thin_combined_traversal::{BoundaryCrossing, ThinCombinedBoundaryTraversal};

mod float_utils;
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
    let mut pixel_traversal = ThinCombinedPixelTraversal::new(ray);

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

pub fn max_z_fast<M: Matrix<Item = f32>, T: Float>(matrix: &M, ray: Ray<T>) -> f32 {
    let mut v = CombinedBoundaryTraversalVariables::new(ray);
    let x_crossings = integers_between(ray.start_x, ray.end_x());
    let y_crossings = integers_between(ray.start_y, ray.end_y());
    let crossings = x_crossings + y_crossings;

    let mut max_z = matrix.get(v.pixel_x as usize, v.pixel_y as usize);
    for _ in 0..crossings {
        if v.t_max_x <= v.t_max_y {
            v.step_x();
        } else {
            v.step_y();
        }

        let z = matrix.get(v.pixel_x as usize, v.pixel_y as usize);
        max_z = max_z.max(z);
    }
    max_z
}

#[inline(never)]
fn hot_loop<T: Float>(mut v: CombinedBoundaryTraversalVariables<T>, crossings: usize, mut callback: impl FnMut(usize, usize)) {
    for _ in 0..(crossings >> 4) {
        repeat!(16 {
            if v.t_max_x <= v.t_max_y {
                v.step_x();
            } else {
                v.step_y();
            }

            (callback)(v.pixel_x as usize, v.pixel_y as usize);
        });
    }

    for _ in 0..(crossings & 0b1111) {
        if v.t_max_x <= v.t_max_y {
            v.step_x();
        } else {
            v.step_y();
        }

        (callback)(v.pixel_x as usize, v.pixel_y as usize);
    }
}

pub fn max_z_fast_variant<M: Matrix<Item = f32>, T: Float>(matrix: &M, ray: Ray<T>) -> f32 {
    let mut v = CombinedBoundaryTraversalVariables::new(ray);
    let x_crossings = integers_between(ray.start_x, ray.end_x());
    let y_crossings = integers_between(ray.start_y, ray.end_y());
    let crossings = x_crossings + y_crossings;

    let mut max_z = matrix.get(v.pixel_x as usize, v.pixel_y as usize);
    hot_loop(v, crossings, #[inline(always)] |x, y| max_z = max_z.max(matrix.get(x, y)));
    max_z

    /*let mut traversal = ThinCombinedBoundaryTraversal::new(ray);
    let first_z = matrix.get(traversal.pixel_x() as usize, traversal.pixel_y() as usize);
    traversal.map(|crossing| {
        let (x, y) = match crossing {
            BoundaryCrossing::X { next_x_index, y_index, .. } => (next_x_index, y_index),
            BoundaryCrossing::Y { x_index, next_y_index, .. } => (x_index, next_y_index),
            BoundaryCrossing::XY { next_x_index, next_y_index, .. } => (next_x_index, next_y_index),
        };
        matrix.get(x as usize, y as usize)
    }).fold(first_z, |a, b| a.max(b))*/
}

#[inline(never)]
pub fn max_z_with_t<M: Matrix<Item = f32>, T: Float>(matrix: &M, ray: Ray<T>) -> Option<f32> {
    let mut pixel_traversal = ThinCombinedPixelTraversal::new(ray);

    pixel_traversal
        .map(|segment| matrix.get(segment.pixel_x as usize, segment.pixel_y as usize))
        .reduce(|a, b| a.max(b))
}

#[inline(never)]
pub fn max_z_without_t<M: Matrix<Item = f32>, T: Float>(matrix: &M, ray: Ray<T>) -> Option<f32> {
    let mut pixel_traversal = ThinCombinedPixelTraversalWithoutT::new(ray);

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
            y_traversal.into_iter().flatten().map(|crossing| {
                matrix.get(crossing.x_index as usize, crossing.next_y_index as usize)
            }),
        )
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
    black_box(max_z_with_t(&matrix, Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 1.0,
        diff_y: 1.0,
    }));
    black_box(max_z_without_t(&matrix, Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 1.0,
        diff_y: 1.0,
    }));
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
                // let mut max_z = max_z_fast(&matrix, ray);
                let mut max_z = max_z_fast_variant(&matrix, ray);
                // let mut max_z = max_z(&matrix, ray).unwrap();
                // let mut max_z = max_z_with_t(&matrix, ray).unwrap();
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
