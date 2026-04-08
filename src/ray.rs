#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray {
    pub start_x: f32,
    pub start_y: f32,
    pub diff_x: f32,
    pub diff_y: f32,
}

impl Ray {
    pub fn end_x(&self) -> f32 {
        self.start_x + self.diff_x
    }
    
    pub fn end_y(&self) -> f32 {
        self.start_y + self.diff_y
    }
}