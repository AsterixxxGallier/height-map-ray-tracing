use crate::ray::Ray;

#[cfg(test)]
mod tests;

// A Fast Voxel Traversal Algorithm by Amanatides and Woo
// implemented to first cross x-boundaries when perfectly meeting corners
pub struct CombinedBoundaryTraversal {
    step_x: isize,
    step_y: isize,
    pixel_x: isize,
    pixel_y: isize,
    t_delta_x: f32,
    t_delta_y: f32,
    t_max_x: f32,
    t_max_y: f32,
}

impl CombinedBoundaryTraversal {
    pub fn new(ray: Ray) -> Self {
        let step_x = ray.diff_x.signum() as isize;
        let step_y = ray.diff_y.signum() as isize;
        let pixel_x = ray.start_x as isize;
        let pixel_y = ray.start_y as isize;

        // How far along the ray we must move for the horizontal component of the movement to equal
        // the width of a pixel (i.e. 1).
        let t_delta_x = ray.diff_x.recip().abs();
        let t_delta_y = ray.diff_y.recip().abs();

        // absolute difference between start_x and the next horizontal pixel boundary
        let dist_x = if ray.diff_x > 0.0 {
            // for x-increasing rays, the next horizontal pixel boundary is at start_x.ceil()
            ray.start_x.ceil() - ray.start_x
        } else {
            // for x-decreasing rays, the next horizontal pixel boundary is at start_x.floor()
            ray.start_x - ray.start_x.floor()
        };
        // absolute difference between start_y and the next vertical pixel boundary
        let dist_y = if ray.diff_y > 0.0 {
            // for y-increasing rays, the next vertical pixel boundary is at start_y.ceil()
            ray.start_y.ceil() - ray.start_y
        } else {
            // for y-decreasing rays, the next vertical pixel boundary is at start_y.floor()
            ray.start_y - ray.start_y.floor()
        };

        // the value of t at which the next vertical pixel boundary is crossed
        // = the value of t at which x is maximal before crossing over the next vertical pixel boundary
        let t_max_x = if t_delta_x.is_finite() {
            t_delta_x * dist_x
        } else {
            f32::INFINITY
        };
        // the value of t at which the next horizontal pixel boundary is crossed
        // = the value of t at which y is maximal before crossing over the next horizontal pixel boundary
        let t_max_y = if t_delta_y.is_finite() {
            t_delta_y * dist_y
        } else {
            f32::INFINITY
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
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BoundaryCrossing {
    X {
        t: f32,
        last_x_index: isize,
        next_x_index: isize,
        y_index: isize,
    },
    Y {
        t: f32,
        x_index: isize,
        last_y_index: isize,
        next_y_index: isize,
    },
}

impl Iterator for CombinedBoundaryTraversal {
    type Item = BoundaryCrossing;

    fn next(&mut self) -> Option<Self::Item> {
        if self.t_max_x <= self.t_max_y {
            // we cross over the next vertical pixel boundary first
            let t = self.t_max_x;
            let last_x_index = self.pixel_x;

            self.t_max_x += self.t_delta_x;
            self.pixel_x += self.step_x;

            let next_x_index = self.pixel_x;
            let y_index = self.pixel_y;

            Some(BoundaryCrossing::X {
                t,
                last_x_index,
                next_x_index,
                y_index,
            })
        } else {
            // we cross over the next horizontal pixel boundary first
            let t = self.t_max_y;
            let last_y_index = self.pixel_y;

            self.t_max_y += self.t_delta_y;
            self.pixel_y += self.step_y;

            let next_y_index = self.pixel_y;
            let x_index = self.pixel_x;

            Some(BoundaryCrossing::Y {
                t,
                x_index,
                last_y_index,
                next_y_index,
            })
        }
    }
}
