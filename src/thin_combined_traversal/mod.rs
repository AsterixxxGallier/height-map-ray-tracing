use std::cmp::Ordering;
use num_traits::ConstOne;
use num_traits::float::Float;
use crate::bounds::Bounds;
use crate::ray::Ray;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct ThinCombinedBoundaryTraversal<T> {
    step_x: i32,
    step_y: i32,
    pixel_x: i32,
    pixel_y: i32,
    t_delta_x: T,
    t_delta_y: T,
    t_max_x: T,
    t_max_y: T,
}

impl<T: Float> ThinCombinedBoundaryTraversal<T> {
    pub fn new(ray: Ray<T>) -> Self {
        let step_x = ray.diff_x.signum().to_i32().unwrap();
        let step_y = ray.diff_y.signum().to_i32().unwrap();

        // the x-index of the pixel that the ray starts in
        // if start_x is an integer, this depends on the x-direction of the ray:
        // - for x-increasing rays, start_x is interpreted to be on the x-upper side of the
        //   x-boundary, so pixel_x = start_x
        // - for x-decreasing rays, start_x is interpreted to be on the x-lower side of the
        //   x-boundary, so pixel_x = start_x - 1
        let pixel_x = if ray.diff_x >= T::zero() {
            // for x-increasing rays, this is simply start_x.floor()
            ray.start_x.to_i32().unwrap()
        } else {
            // for x-decreasing rays, this is start_x - 1 if start_x is an integer, and
            // start_x.floor() otherwise
            ray.start_x.ceil().to_i32().unwrap() - 1
        };
        // the y-index of the pixel that the ray starts in
        // if start_y is an integer, this depends on the y-direction of the ray:
        // - for y-increasing rays, start_y is interpreted to be on the y-upper side of the
        //   y-boundary, so pixel_y = start_y
        // - for y-decreasing rays, start_y is interpreted to be on the y-lower side of the
        //   y-boundary, so pixel_y = start_y - 1
        let pixel_y = if ray.diff_y >= T::zero() {
            // for y-increasing rays, this is simply start_y.floor()
            ray.start_y.to_i32().unwrap()
        } else {
            // for y-decreasing rays, this is start_y - 1 if start_y is an integer, and
            // start_y.floor() otherwise
            ray.start_y.ceil().to_i32().unwrap() - 1
        };

        // difference in t that corresponds to a difference in x of exactly 1.0
        let t_delta_x = ray.diff_x.recip().abs();
        // difference in t that corresponds to a difference in y of exactly 1.0
        let t_delta_y = ray.diff_y.recip().abs();

        // absolute difference between start_x and the next x-boundary
        let dist_x = if ray.diff_x >= T::zero() {
            // If start_x is an integer, this is just 1.
            // Otherwise, this is start_x.ceil() - start_x (the difference to the next-up integer).
            ray.start_x.floor() - ray.start_x + T::one()
        } else {
            // If start_x is an integer, this is just 1.
            // Otherwise, this is start_x - start_x.floor() (the difference to the next-down integer).
            ray.start_x - ray.start_x.ceil() + T::one()
        };
        // absolute difference between start_y and the next y-boundary
        let dist_y = if ray.diff_y >= T::zero() {
            // If start_y is an integer, this is just 1.
            // Otherwise, this is start_y.ceil() - start_y (the difference to the next-up integer).
            ray.start_y.floor() - ray.start_y + T::one()
        } else {
            // If start_y is an integer, this is just 1.
            // Otherwise, this is start_y - start_y.floor() (the difference to the next-down integer).
            ray.start_y - ray.start_y.ceil() + T::one()
        };

        // the value of t at which the next x-boundary is crossed
        // = the value of t at which x is maximal before crossing over the next x-boundary
        let t_max_x = if t_delta_x.is_finite() {
            t_delta_x * dist_x
        } else {
            T::infinity()
        };
        // the value of t at which the next y-boundary is crossed
        // = the value of t at which y is maximal before crossing over the next y-boundary
        let t_max_y = if t_delta_y.is_finite() {
            t_delta_y * dist_y
        } else {
            T::infinity()
        };

        Self {
            step_x,
            step_y,
            pixel_x,
            pixel_y,
            t_delta_x,
            t_delta_y,
            t_max_x,
            t_max_y,
        }
    }

    pub fn pixel_x(&self) -> i32 {
        self.pixel_x
    }

    pub fn pixel_y(&self) -> i32 {
        self.pixel_y
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
        let t = self.t_max_x.min(self.t_max_y);

        if t >= T::from(0.999999f32).unwrap() {
            return None;
        }

        if self.t_max_x < self.t_max_y {
            // we cross over the next x-boundary first
            let last_x_index = self.pixel_x;

            self.t_max_x = self.t_max_x + self.t_delta_x;
            self.pixel_x += self.step_x;

            let next_x_index = self.pixel_x;
            let y_index = self.pixel_y;

            Some(BoundaryCrossing::X {
                t,
                last_x_index,
                next_x_index,
                y_index,
            })
        } else if self.t_max_x > self.t_max_y {
            // we cross over the next y-boundary first
            let last_y_index = self.pixel_y;

            self.t_max_y = self.t_max_y + self.t_delta_y;
            self.pixel_y += self.step_y;

            let next_y_index = self.pixel_y;
            let x_index = self.pixel_x;

            Some(BoundaryCrossing::Y {
                t,
                x_index,
                last_y_index,
                next_y_index,
            })
        } else {
            // we cross over the next x-boundary and the next y-boundary simultaneously
            let last_x_index = self.pixel_x;
            let last_y_index = self.pixel_y;

            self.t_max_x = self.t_max_x + self.t_delta_x;
            self.t_max_y = self.t_max_y + self.t_delta_y;
            self.pixel_x += self.step_x;
            self.pixel_y += self.step_y;

            let next_y_index = self.pixel_y;
            let next_x_index = self.pixel_x;

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
