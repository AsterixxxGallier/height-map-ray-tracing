use crate::float_utils::{integers_between, BoundaryTraversalVariables};
use crate::ray::Ray;
use num_traits::float::Float;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct BoundaryTraversal<T> {
    v: BoundaryTraversalVariables<T>,
    remaining_crossings: usize,
}

impl<T: Float> BoundaryTraversal<T> {
    pub fn new(ray: Ray<T>) -> Self {
        let x_crossings = integers_between(ray.start_x, ray.end_x());
        let y_crossings = integers_between(ray.start_y, ray.end_y());
        let total_crossings = x_crossings + y_crossings;

        Self {
            v: BoundaryTraversalVariables::new(ray),
            remaining_crossings: total_crossings,
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
pub enum BoundaryType {
    X,
    Y,
    XY,
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

        if self.v.t_max_x < self.v.t_max_y {
            if self.remaining_crossings == 0 {
                return None;
            }

            self.remaining_crossings -= 1;

            self.v.step_x();

            Some(BoundaryCrossing {
                boundary_type: BoundaryType::X,
                t,
                pixel_x: self.v.pixel_x,
                pixel_y: self.v.pixel_y,
            })
        } else if self.v.t_max_y < self.v.t_max_x {
            if self.remaining_crossings == 0 {
                return None;
            }

            self.remaining_crossings -= 1;

            self.v.step_y();

            Some(BoundaryCrossing {
                boundary_type: BoundaryType::Y,
                t,
                pixel_x: self.v.pixel_x,
                pixel_y: self.v.pixel_y,
            })
        } else {
            if self.remaining_crossings <= 1 {
                return None;
            }

            self.remaining_crossings -= 2;

            self.v.step_x();
            self.v.step_y();

            Some(BoundaryCrossing {
                boundary_type: BoundaryType::XY,
                t,
                pixel_x: self.v.pixel_x,
                pixel_y: self.v.pixel_y,
            })
        }
    }
}
