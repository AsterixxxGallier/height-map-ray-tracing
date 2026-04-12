use num_traits::Float;
use crate::ray::Ray2;
use core::*;

mod core;
#[cfg(test)]
mod tests;

/// Boundary types characterized by which coordinate integer parts change value.
///
/// For example, a ray crossing an x-boundary means that the integer part of its x coordinate
/// changed in value (e.g. from `0.99` to `1.01`).
///
/// `BoundaryType::XY` signals that the integer parts of both the x and y coordinates changed
/// simultaneously, i.e. a corner was crossed.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BoundaryType {
    X,
    Y,
    XY,
}

/// An iterator over the boundaries crossed by a given [ray](Ray2). A boundary is considered crossed
/// if the ray is on both sides of the boundary - merely touching the boundary does not suffice.
///
/// The iteration stops when `t` logically reaches or exceeds `1.0`. Precautions have been taken in
/// the implementation of the iterator to guarantee that floating-point rounding errors can never
/// result in the ray "over-" or "undershooting". Even when reported values of `t` exceed `1.0`, or
/// incorrectly never reach `1.0`, the boundary crossings produced are correct in
/// [type](BoundaryType), order, number and pixel coordinates.
///
/// The underlying algorithm is based on the paper "A Fast Voxel Traversal Algorithm" by Amanatides
/// and Woo, but modified for precision and to account for edge-cases.
#[derive(Debug)]
pub struct BoundaryTraversal<T> {
    v: BoundaryTraversalVariables<T>,
    remaining_x_crossings: usize,
    remaining_y_crossings: usize,
}

impl<T: Float> BoundaryTraversal<T> {
    pub fn new(ray: Ray2<T>) -> Self {
        let x_crossings = integers_between(ray.start_x, ray.end_x());
        let y_crossings = integers_between(ray.start_y, ray.end_y());

        Self {
            v: BoundaryTraversalVariables::new(ray),
            remaining_x_crossings: x_crossings,
            remaining_y_crossings: y_crossings,
        }
    }

    pub fn pixel_x(&self) -> i32 {
        self.v.pixel_x
    }

    pub fn pixel_y(&self) -> i32 {
        self.v.pixel_y
    }
}

/// The event of a ray crossing a boundary, characterized by the [type](BoundaryType) of the crossed
/// boundary, the `t`-value at the crossing, and the coordinates of the pixel that was entered.
///
/// `t` may be imprecise due to floating-point rounding errors, which linearly accumulate over the
/// course of the iteration.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BoundaryCrossing<T> {
    pub boundary_type: BoundaryType,
    pub t: T,
    pub pixel_x: i32,
    pub pixel_y: i32,
}

impl<T: Float> Iterator for BoundaryTraversal<T> {
    type Item = BoundaryCrossing<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let t = self.v.t_max_x.min(self.v.t_max_y);

        let boundary_type = self.v.next_boundary_type();

        match boundary_type {
            BoundaryType::X => {
                self.remaining_x_crossings = self.remaining_x_crossings.checked_sub(1)?;
                self.v.step_x();
            }
            BoundaryType::Y => {
                self.remaining_y_crossings = self.remaining_y_crossings.checked_sub(1)?;
                self.v.step_y();
            }
            BoundaryType::XY => {
                self.remaining_x_crossings = self.remaining_x_crossings.checked_sub(1)?;
                self.remaining_y_crossings = self.remaining_y_crossings.checked_sub(1)?;
                self.v.step_x();
                self.v.step_y();
            }
        }

        Some(BoundaryCrossing {
            boundary_type,
            t,
            pixel_x: self.v.pixel_x,
            pixel_y: self.v.pixel_y,
        })
    }
}
