use crate::ray::Ray;
use num_traits::Float;
use crate::float_utils::{distance_to_integer_boundary, initial_pixel_coordinate, integers_between, CombinedBoundaryTraversalVariables};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct CombinedBoundaryTraversal<T> {
    v: CombinedBoundaryTraversalVariables<T>,
    n_iters: usize,
}

impl<T: Float> CombinedBoundaryTraversal<T> {
    pub fn new(ray: Ray<T>) -> Self {
        let x_crossings = integers_between(ray.start_x, ray.end_x());
        let y_crossings = integers_between(ray.start_y, ray.end_y());
        let n_iters = x_crossings + y_crossings;

        Self {
            v: CombinedBoundaryTraversalVariables::new(ray),
            n_iters,
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
pub enum BoundaryCrossing<T> {
    X {
        t: T,
        last_x_index: i32,
        next_x_index: i32,
        y_index: i32,
    },
    Y {
        t: T,
        x_index: i32,
        last_y_index: i32,
        next_y_index: i32,
    },
}

impl<T: Float> Iterator for CombinedBoundaryTraversal<T> {
    type Item = BoundaryCrossing<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n_iters == 0 {
            return None;
        }

        self.n_iters -= 1;

        let t = self.v.t_max_x.min(self.v.t_max_y);

        if self.v.t_max_x <= self.v.t_max_y {
            // we cross over the next x-boundary first
            let last_x_index = self.v.pixel_x;

            self.v.step_x();

            let next_x_index = self.v.pixel_x;
            let y_index = self.v.pixel_y;

            Some(BoundaryCrossing::X {
                t,
                last_x_index,
                next_x_index,
                y_index,
            })
        } else {
            // we cross over the next y-boundary first
            let last_y_index = self.v.pixel_y;

            self.v.step_y();

            let next_y_index = self.v.pixel_y;
            let x_index = self.v.pixel_x;

            Some(BoundaryCrossing::Y {
                t,
                x_index,
                last_y_index,
                next_y_index,
            })
        }
    }
}
