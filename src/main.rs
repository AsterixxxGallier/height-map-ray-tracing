#![allow(unused)]

use crate::combined_pixel_traversal::CombinedPixelTraversal;
use crate::matrix::Matrix;
use crate::ray_z::RayZ;

pub mod bounds;
pub mod combined_pixel_traversal;
pub mod combined_traversal;
pub mod thin_combined_traversal;
pub mod in_bounds_combined_traversal;
pub mod matrix;
pub mod ray;
pub mod ray_z;
pub mod separate_traversal;
#[cfg(test)]
mod tests;

pub fn is_line_free<M: Matrix<Item = f32>>(matrix: &M, ray_z: RayZ) -> bool {
    let ray = ray_z.as_ray();
    let mut pixel_traversal = CombinedPixelTraversal::new(ray);

    if ray_z.diff_z >= 0.0 {
        pixel_traversal.all(|segment| {
            matrix.get(segment.pixel_x as usize, segment.pixel_y as usize)
                < (ray_z.start_z + segment.start_t * ray_z.diff_z)
        })
    } else {
        pixel_traversal.all(|segment| {
            matrix.get(segment.pixel_x as usize, segment.pixel_y as usize)
                < ray_z.start_z + segment.end_t * ray_z.diff_z
        })
    }
}

fn main() {}
