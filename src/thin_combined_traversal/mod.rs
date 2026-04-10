use crate::float_utils::CombinedBoundaryTraversalVariables;
use crate::ray::Ray;
use num_traits::float::Float;
use num_traits::ConstOne;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct ThinCombinedBoundaryTraversal<T> {
    v: CombinedBoundaryTraversalVariables<T>,
}

impl<T: Float> ThinCombinedBoundaryTraversal<T> {
    pub fn new(ray: Ray<T>) -> Self {
        Self {
            v: CombinedBoundaryTraversalVariables::new(ray),
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
    XY {
        t: T,
        last_x_index: i32,
        next_x_index: i32,
        last_y_index: i32,
        next_y_index: i32,
    },
}

impl<T: Float> Iterator for ThinCombinedBoundaryTraversal<T> {
    type Item = BoundaryCrossing<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let t = self.v.t_max_x.min(self.v.t_max_y);

        if t >= T::from(0.999999f32).unwrap() {
            return None;
        }

        if self.v.t_max_x < self.v.t_max_y {
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
        } else if self.v.t_max_x > self.v.t_max_y {
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
        } else {
            // we cross over the next x-boundary and the next y-boundary simultaneously
            let last_x_index = self.v.pixel_x;
            let last_y_index = self.v.pixel_y;

            self.v.step_x();
            self.v.step_y();

            let next_y_index = self.v.pixel_y;
            let next_x_index = self.v.pixel_x;

            Some(BoundaryCrossing::XY {
                t,
                last_x_index,
                next_x_index,
                last_y_index,
                next_y_index,
            })
        }
    }
}
