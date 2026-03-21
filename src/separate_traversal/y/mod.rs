use crate::ray::Ray;

pub struct YBoundaryTraversal {
    step_x: f32,
    step_y: f32,
    t_delta_y: f32,
    x: f32,
    y: f32,
    t: f32,
}

impl YBoundaryTraversal {
    /// Returns a new instance if `ray.diff_y != 0`.
    pub fn new(ray: Ray) -> Option<Self> {
        // whether the ray moves y-increasing or y-decreasing
        let step_y = ray.diff_y.signum();
        // how much x-movement happens when we move by one pixel in the y-direction
        let step_x = ray.diff_x / ray.diff_y;
        // how far along the ray we have to move for the y-component of the movement to equal 1.0
        let t_delta_y = ray.diff_y.recip();
        let mut x = ray.start_x;
        let mut y = ray.start_y;
        let mut t = 0.0;

        if step_y == 0.0 {
            return None;
        }

        if y.fract() != 0.0 {
            // find the first y-boundary that the ray crosses, and position x and y accordingly

            let dist_y = if step_y > 0.0 {
                y.ceil() - y
            } else {
                y - y.floor()
            };

            x += dist_y * step_x;
            y += dist_y * step_y;
            t += dist_y * t_delta_y;
        } else {
            // the ray starts at an y-boundary
        }

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
pub struct YBoundaryCrossing {
    t: f32,
    x_index: isize,
    last_y_index: isize,
    next_y_index: isize,
}

impl Iterator for YBoundaryTraversal {
    type Item = YBoundaryCrossing;

    fn next(&mut self) -> Option<Self::Item> {
        let item = YBoundaryCrossing {
            t: self.t,
            x_index: self.x as isize,
            last_y_index: self.y as isize - 1,
            next_y_index: self.y as isize,
        };

        self.x += self.step_x;
        self.y += self.step_y;
        self.t += self.t_delta_y;

        Some(item)
    }
}