use std::ops::Add;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray2<T> {
    pub start_x: T,
    pub start_y: T,
    pub diff_x: T,
    pub diff_y: T,
}

impl<T: Copy + Add<Output=T>> Ray2<T> {
    pub fn end_x(&self) -> T {
        self.start_x + self.diff_x
    }
    
    pub fn end_y(&self) -> T {
        self.start_y + self.diff_y
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray3<T> {
    pub start_x: T,
    pub start_y: T,
    pub start_z: T,
    pub diff_x: T,
    pub diff_y: T,
    pub diff_z: T,
}

impl<T: Copy + Add<Output=T>> Ray3<T> {
    pub fn end_x(&self) -> T {
        self.start_x + self.diff_x
    }

    pub fn end_y(&self) -> T {
        self.start_y + self.diff_y
    }

    pub fn end_z(&self) -> T {
        self.start_z + self.diff_z
    }

    pub fn as_ray_2(&self) -> Ray2<T> {
        Ray2 {
            start_x: self.start_x,
            start_y: self.start_y,
            diff_x: self.diff_x,
            diff_y: self.diff_y,
        }
    }
}