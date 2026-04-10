use crate::map::{ArrayMap, Map};
use traversal::pixel::PixelTraversal;
use crate::ray::{Ray2, Ray3};
use image::{Rgb, RgbImage};
use num_traits::Float;
use rayon::iter::ParallelIterator;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator};
use std::f64::consts::PI;
use std::fs::File;
use std::time::Instant;
use tiff::decoder::{Decoder, DecodingResult};

pub mod traversal;
pub mod map;
pub mod ray;

pub fn is_line_free<M: Map<Item = f32>, T: Float>(map: &M, ray_3: Ray3<T>) -> bool {
    let ray_2 = ray_3.as_ray_2();
    let mut pixel_traversal = PixelTraversal::new(ray_2);

    if ray_3.diff_z >= T::zero() {
        pixel_traversal.all(|segment| {
            map.get(segment.pixel_x as usize, segment.pixel_y as usize)
                < (ray_3.start_z + segment.start_t * ray_3.diff_z)
                    .to_f32()
                    .unwrap()
        })
    } else {
        pixel_traversal.all(|segment| {
            map.get(segment.pixel_x as usize, segment.pixel_y as usize)
                < (ray_3.start_z + segment.end_t * ray_3.diff_z)
                    .to_f32()
                    .unwrap()
        })
    }
}

pub fn max_z<M: Map<Item = f32>, T: Float>(map: &M, ray: Ray2<T>) -> Option<f32> {
    PixelTraversal::new(ray)
        .map(|segment| map.get(segment.pixel_x as usize, segment.pixel_y as usize))
        .reduce(|a, b| a.max(b))
}

fn main() {
    let file = File::open("LHD_FXX_0648_6863_MNS_O_0M50_LAMB93_IGN69.tif").unwrap();
    let io = std::io::BufReader::new(file);
    let mut reader = Decoder::new(io).unwrap();

    let mut data = DecodingResult::F32(vec![]);

    _ = reader.read_image_to_buffer(&mut data).unwrap();

    let DecodingResult::F32(data) = data else {
        panic!()
    };

    let x_size = 2000;
    let y_size = 2000;
    let z_size = 100;

    let map = ArrayMap::from_vec(x_size, y_size, data);
    map.save_as_image(100.0, "map.png");

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
        "{} rays computed in {:?} ({:.2} million rays per second, {:.2} fs per ray pixel, {:.2} trillion ray pixels per second)",
        num_rays,
        elapsed,
        num_rays as f64 / elapsed.as_secs_f64() / 1e6,
        elapsed.as_nanos() as f64 / (num_rays as usize * map.len()) as f64 * 1e6,
        num_rays as f64 * map.len() as f64 / elapsed.as_secs_f64() / 1e12,
    );
    image.save("out.png").unwrap();
}
