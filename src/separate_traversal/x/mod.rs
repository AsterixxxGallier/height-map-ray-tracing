use crate::ray::Ray;
use num_traits::Float;
use std::fmt::Debug;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct XBoundaryTraversal<T> {
    step_x: T,
    step_y: T,
    t_delta_x: T,
    x: T,
    y: T,
    t: T,
}

impl<T: Float> XBoundaryTraversal<T> {
    /// Returns a new instance if `ray.diff_x != 0`.
    pub fn new(ray: Ray<T>) -> Option<Self> {
        // whether the ray moves x-increasing or x-decreasing
        let step_x = ray.diff_x.signum();
        // how much y-movement happens when we move by one pixel in the x-direction
        let step_y = ray.diff_y / ray.diff_x;
        // how far along the ray we have to move for the x-component of the movement to equal 1.0
        let t_delta_x = ray.diff_x.recip().abs();
        let mut x = ray.start_x;
        let mut y = ray.start_y;
        let mut t = T::zero();

        if step_x == T::zero() {
            return None;
        }

        // This is the x-distance to the first boundary crossing.
        let dist_x = if step_x > T::zero() {
            // If x is an integer, this is just 1.
            // Otherwise, this is x.ceil() - x (the difference to the next-up integer).
            x.floor() - x + T::one()
        } else {
            // If x is an integer, this is just 1.
            // Otherwise, this is x - x.floor() (the difference to the next-down integer).
            x - x.ceil() + T::one()
        };

        // Move by dist_x along the ray.
        x = x + dist_x * step_x;
        y = y + dist_x * step_y;
        t = t + dist_x * t_delta_x;

        Some(Self {
            step_x,
            step_y,
            t_delta_x,
            x,
            y,
            t,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct XBoundaryCrossing<T> {
    pub t: T,
    pub last_x_index: i32,
    pub next_x_index: i32,
    pub y_index: i32,
}

impl<T: Float> Iterator for XBoundaryTraversal<T> {
    type Item = XBoundaryCrossing<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.t >= T::from(0.999999f32).unwrap() {
            return None;
        }

        let item = unsafe {
            XBoundaryCrossing {
                t: self.t,
                last_x_index: (self.x - self.step_x).to_i32().unwrap_unchecked(),
                next_x_index: self.x.to_i32().unwrap_unchecked(),
                y_index: self.y.to_i32().unwrap_unchecked(),
            }
        };

        self.x = self.x + self.step_x;
        self.y = self.y + self.step_y;
        self.t = self.t + self.t_delta_x;

        Some(item)
    }
}
