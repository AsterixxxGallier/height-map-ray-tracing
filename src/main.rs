#![allow(unused)]

use crate::combined_pixel_traversal::CombinedPixelTraversal;
use crate::matrix::{ArrayMatrix, Matrix};
use crate::ray::Ray;

pub mod bounds;
pub mod combined_pixel_traversal;
pub mod combined_traversal;
pub mod in_bounds_combined_traversal;
pub mod matrix;
pub mod ray;
pub mod separate_traversal;

pub fn is_line_free<M: Matrix<Item = f32>>(
    matrix: &M,
    start_x: f32,
    start_y: f32,
    start_z: f32,
    end_x: f32,
    end_y: f32,
    end_z: f32,
) -> bool {
    let ray = Ray {
        start_x,
        start_y,
        diff_x: end_x - start_x,
        diff_y: end_y - start_y,
    };
    let mut pixel_traversal = CombinedPixelTraversal::new(ray);

    let diff_z = end_z - start_z;
    if diff_z >= 0.0 {
        !pixel_traversal.any(|segment| {
            matrix.get(segment.pixel_x as usize, segment.pixel_y as usize)
                >= start_z + segment.start_t * diff_z
        })
    } else {
        !pixel_traversal.any(|segment| {
            matrix.get(segment.pixel_x as usize, segment.pixel_y as usize)
                >= start_z + segment.end_t * diff_z
        })
    }
}

fn main() {
    let mut matrix = ArrayMatrix::<f32>::new(3, 3);
    matrix.set(1, 1, 1.0);

    let free = is_line_free(&matrix, 0.0, 0.0, 0.5, 3.0, 3.0, 0.5);
    assert_eq!(free, false);

    let free = is_line_free(&matrix, 0.0, 0.0, 0.5, 3.0, 1.0, 0.5);
    assert_eq!(free, true);

    let free = is_line_free(&matrix, 0.0, 0.0, 0.5, 3.0, 2.0, 0.5);
    assert_eq!(free, false);

    let free = is_line_free(&matrix, 0.0, 1.5, 2.0, 3.0, 1.5, 0.0);
    assert_eq!(free, false);
}
