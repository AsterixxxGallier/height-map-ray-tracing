use num_traits::Float;
use crate::ray::Ray2;
use core::*;

mod core;
#[cfg(test)]
mod tests;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BoundaryType {
    X,
    Y,
    XY,
}

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
