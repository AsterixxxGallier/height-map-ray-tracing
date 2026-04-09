use std::ops::Add;
use crate::ray::Ray;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RayZ<T> {
    pub start_x: T,
    pub start_y: T,
    pub start_z: T,
    pub diff_x: T,
    pub diff_y: T,
    pub diff_z: T,
}

impl<T: Copy + Add<Output=T>> RayZ<T> {
    pub fn end_x(&self) -> T {
        self.start_x + self.diff_x
    }

    pub fn end_y(&self) -> T {
        self.start_y + self.diff_y
    }

    pub fn end_z(&self) -> T {
        self.start_z + self.diff_z
    }

    pub fn as_ray(&self) -> Ray<T> {
        Ray {
            start_x: self.start_x,
            start_y: self.start_y,
            diff_x: self.diff_x,
            diff_y: self.diff_y,
        }
    }
}