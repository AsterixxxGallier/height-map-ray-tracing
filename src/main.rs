#![allow(unused)]

use crate::matrix::{isize_indices_in_matrix_bounds, ArrayMatrix, Matrix};
use crate::ray::Ray;

pub mod separate_traversal;
pub mod matrix;
pub mod ray;
pub mod combined_traversal;
pub mod bounds;

pub fn is_line_free<M: Matrix<Item = f32>>(
    matrix: &M,
    start_x: f32,
    start_y: f32,
    start_z: f32,
    end_x: f32,
    end_y: f32,
    end_z: f32,
) -> bool {
    // t ranges from 0 to 1 (inclusive), where t = 0 corresponds to the ray origin and t = 1
    // corresponds to the ray destination.

    let diff_x = end_x - start_x;
    let diff_y = end_y - start_y;
    let diff_z = end_z - start_z;
    let step_x = diff_x.signum() as isize;
    let step_y = diff_y.signum() as isize;
    let pixel_x = start_x as isize;
    let pixel_y = start_y as isize;

    // How far along the ray we must move for the horizontal component of the movement to equal the
    // width of a pixel (i.e. 1).
    let t_delta_x = diff_x.recip();
    let t_delta_y = diff_y.recip();

    // absolute difference between start_x and the next horizontal pixel boundary
    let dist_x = if diff_x > 0.0 {
        // for x-increasing rays, the next horizontal pixel boundary is at start_x.ceil()
        start_x.ceil() - start_x
    } else {
        // for x-decreasing rays, the next horizontal pixel boundary is at start_x.floor()
        start_x - start_x.floor()
    };
    // absolute difference between start_y and the next vertical pixel boundary
    let dist_y = if diff_y > 0.0 {
        // for y-increasing rays, the next vertical pixel boundary is at start_y.ceil()
        start_y.ceil() - start_y
    } else {
        // for y-decreasing rays, the next vertical pixel boundary is at start_y.floor()
        start_y - start_y.floor()
    };

    // the value of t at which the next vertical pixel boundary is crossed
    // = the value of t at which x is maximal before crossing over the next vertical pixel boundary
    let t_max_x = if t_delta_x.is_finite() {
        t_delta_x * dist_x
    } else {
        f32::INFINITY
    };
    // the value of t at which the next horizontal pixel boundary is crossed
    // = the value of t at which y is maximal before crossing over the next horizontal pixel boundary
    let t_max_y = if t_delta_y.is_finite() {
        t_delta_y * dist_y
    } else {
        f32::INFINITY
    };

    let mut pixel_x = pixel_x;
    let mut pixel_y = pixel_y;
    let mut t_max_x = t_max_x;
    let mut t_max_y = t_max_y;
    let mut z = start_z;
    let mut t = 0.0;

    while isize_indices_in_matrix_bounds(matrix, pixel_x, pixel_y) {
        println!("t = {t}, z = {z}, pixel_x = {pixel_x}, pixel_y = {pixel_y}");

        if t_max_x < t_max_y {
            // we cross over the next vertical pixel boundary first
            t = t_max_x;
            z = start_z + t * diff_z;

            // get the maximum of matrix values at the pixels to both sides of the boundary we cross
            let max_height = matrix
                .get(pixel_x as usize, pixel_y as usize)
                .max(matrix.get((pixel_x + step_x) as usize, pixel_y as usize));
            if max_height > z {
                println!("collision at {pixel_x} {pixel_y}");
                return false;
            }

            t_max_x += t_delta_x;
            pixel_x += step_x;
        } else {
            // we cross over the next horizontal pixel boundary first
            t = t_max_y;
            z = start_z + t * diff_z;

            // get the maximum of matrix values at the pixels to both sides of the boundary we cross
            let max_height = matrix
                .get(pixel_x as usize, pixel_y as usize)
                .max(matrix.get(pixel_x as usize, (pixel_y + step_y) as usize));
            if max_height > z {
                println!("collision at {pixel_x} {pixel_y}");
                return false;
            }

            t_max_y += t_delta_y;
            pixel_y += step_y;
        }
    }

    true
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

    println!(".\n.\n.\n.");
    let free = is_line_free(&matrix, 0.0, 1.5, 2.0, 3.0, 1.5, 0.0);
    assert_eq!(free, false);
}
