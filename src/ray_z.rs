use crate::ray::Ray;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RayZ {
    pub start_x: f32,
    pub start_y: f32,
    pub start_z: f32,
    pub diff_x: f32,
    pub diff_y: f32,
    pub diff_z: f32,
}

impl RayZ {
    pub fn end_x(&self) -> f32 {
        self.start_x + self.diff_x
    }

    pub fn end_y(&self) -> f32 {
        self.start_y + self.diff_y
    }

    pub fn end_z(&self) -> f32 {
        self.start_z + self.diff_z
    }

    pub fn as_ray(&self) -> Ray {
        Ray {
            start_x: self.start_x,
            start_y: self.start_y,
            diff_x: self.diff_x,
            diff_y: self.diff_y,
        }
    }
}