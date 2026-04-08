use crate::ray::Ray;

#[cfg(test)]
mod tests;

pub struct XBoundaryTraversal {
    step_x: f32,
    step_y: f32,
    t_delta_x: f32,
    x: f32,
    y: f32,
    t: f32,
}

impl XBoundaryTraversal {
    /// Returns a new instance if `ray.diff_x != 0`.
    pub fn new(ray: Ray) -> Option<Self> {
        // whether the ray moves x-increasing or x-decreasing
        let step_x = ray.diff_x.signum();
        // how much y-movement happens when we move by one pixel in the x-direction
        let step_y = ray.diff_y / ray.diff_x;
        // how far along the ray we have to move for the x-component of the movement to equal 1.0
        let t_delta_x = ray.diff_x.recip().abs();
        let mut x = ray.start_x;
        let mut y = ray.start_y;
        let mut t = 0.0;

        if step_x == 0.0 {
            return None;
        }

        // This is the x-distance to the first boundary crossing.
        let dist_x = if step_x > 0.0 {
            // If x is an integer, this is just 1.
            // Otherwise, this is x.ceil() - x (the difference to the next-up integer).
            x.floor() - x + 1.0
        } else {
            // If x is an integer, this is just 1.
            // Otherwise, this is x - x.floor() (the difference to the next-down integer).
            x - x.ceil() + 1.0
        };

        // Move by dist_x along the ray.
        x += dist_x * step_x;
        y += dist_x * step_y;
        t += dist_x * t_delta_x;

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
pub struct XBoundaryCrossing {
    t: f32,
    last_x_index: isize,
    next_x_index: isize,
    y_index: isize,
}

impl Iterator for XBoundaryTraversal {
    type Item = XBoundaryCrossing;

    fn next(&mut self) -> Option<Self::Item> {
        let item = XBoundaryCrossing {
            t: self.t,
            last_x_index: (self.x - self.step_x) as isize,
            next_x_index: self.x as isize,
            y_index: self.y as isize,
        };

        self.x += self.step_x;
        self.y += self.step_y;
        self.t += self.t_delta_x;

        Some(item)
    }
}