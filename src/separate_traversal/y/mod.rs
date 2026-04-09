use crate::ray::Ray;
use num_traits::Float;
use std::fmt::Debug;

pub struct YBoundaryTraversal<T> {
    step_x: T,
    step_y: T,
    t_delta_y: T,
    x: T,
    y: T,
    t: T,
}

impl<T: Float> YBoundaryTraversal<T> {
    /// Returns a new instance if `ray.diff_y != 0`.
    pub fn new(ray: Ray<T>) -> Option<Self> {
        // whether the ray moves y-increasing or y-decreasing
        let step_y = ray.diff_y.signum();
        // how much x-movement happens when we move by one pixel in the y-direction
        let step_x = ray.diff_x / ray.diff_y;
        // how far along the ray we have to move for the y-component of the movement to equal 1.0
        let t_delta_y = ray.diff_y.recip().abs();
        let mut x = ray.start_x;
        let mut y = ray.start_y;
        let mut t = T::zero();

        if step_y == T::zero() {
            return None;
        }

        // This is the y-distance to the first boundary crossing.
        let dist_y = if step_y > T::zero() {
            // If y is an integer, this is just 1.
            // Otherwise, this is y.ceil() - y (the difference to the next-up integer).
            y.floor() - y + T::one()
        } else {
            // If y is an integer, this is just 1.
            // Otherwise, this is y - y.floor() (the difference to the next-down integer).
            y - y.ceil() + T::one()
        };

        // Move by dist_y along the ray.
        x = x + dist_y * step_x;
        y = y + dist_y * step_y;
        t = t + dist_y * t_delta_y;

        Some(Self {
            step_x,
            step_y,
            t_delta_y,
            x,
            y,
            t,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct YBoundaryCrossing<T> {
    pub t: T,
    pub x_index: i32,
    pub last_y_index: i32,
    pub next_y_index: i32,
}

impl<T: Float> Iterator for YBoundaryTraversal<T> {
    type Item = YBoundaryCrossing<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.t >= T::from(0.999999f32).unwrap() {
            return None;
        }

        let item = unsafe {
            YBoundaryCrossing {
                t: self.t,
                x_index: self.x.to_i32().unwrap_unchecked(),
                last_y_index: (self.y - self.step_y).to_i32().unwrap_unchecked(),
                next_y_index: self.y.to_i32().unwrap_unchecked(),
            }
        };

        self.x = self.x + self.step_x;
        self.y = self.y + self.step_y;
        self.t = self.t + self.t_delta_y;

        Some(item)
    }
}
