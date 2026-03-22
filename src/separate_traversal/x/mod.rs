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
        // TODO: should this be .recip().abs()? (see CombinedBoundaryTraversal; same for XBoun.Tr.)
        let t_delta_x = ray.diff_x.recip();
        let mut x = ray.start_x;
        let mut y = ray.start_y;
        let mut t = 0.0;

        if step_x == 0.0 {
            return None;
        }

        if x.fract() != 0.0 {
            // find the first x-boundary that the ray crosses, and position x and y accordingly

            let dist_x = if step_x > 0.0 {
                x.ceil() - x
            } else {
                x - x.floor()
            };

            x += dist_x * step_x;
            y += dist_x * step_y;
            t += dist_x * t_delta_x;
        } else {
            // the ray starts at an x-boundary
        }

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