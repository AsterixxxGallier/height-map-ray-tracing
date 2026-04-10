use image::{ImageBuffer, Rgb};
use rand::distr::Distribution;
use rand::Rng;

pub struct MaxReducedMatrices(Vec<ArrayMatrix<f32>>);

impl MaxReducedMatrices {
    pub fn new(matrix: ArrayMatrix<f32>) -> Self {
        assert!(matrix.x_len().is_power_of_two());
        assert!(matrix.y_len().is_power_of_two());
        assert_eq!(matrix.x_len(), matrix.y_len());
        let mut vec = vec![matrix];
        while vec.last().unwrap().x_len() > 1 {
            vec.push(vec.first().unwrap().max_reduce());
        }
        Self(vec)
    }

    pub fn get(&self, reduction_level: usize) -> &ArrayMatrix<f32> {
        &self.0[reduction_level]
    }
}

pub trait Matrix {
    type Item;

    fn x_len(&self) -> usize;

    fn y_len(&self) -> usize;

    fn len(&self) -> usize {
        self.x_len() * self.y_len()
    }

    fn get(&self, x_index: usize, y_index: usize) -> Self::Item;

    fn set(&mut self, x_index: usize, y_index: usize, item: Self::Item);
}

/// Stores items in a contiguous array on the heap.
#[derive(Debug)]
pub struct ArrayMatrix<T> {
    store: Box<[T]>,
    x_len: usize,
    y_len: usize,
}

impl<T: Copy + Default> ArrayMatrix<T> {
    pub fn new(x_len: usize, y_len: usize) -> Self {
        let store = vec![T::default(); x_len * y_len].into_boxed_slice();
        Self {
            store,
            x_len,
            y_len,
        }
    }

    pub fn random(
        x_len: usize,
        y_len: usize,
        distribution: impl Distribution<T>,
        rng: &mut impl Rng,
    ) -> Self {
        let store: Box<[T]> = distribution.sample_iter(rng).take(x_len * y_len).collect();
        Self {
            store,
            x_len,
            y_len,
        }
    }

    pub fn from_vec(
        x_len: usize,
        y_len: usize,
        vec: Vec<T>,
    ) -> Self {
        assert_eq!(vec.len(), x_len * y_len);
        Self {
            store: vec.into_boxed_slice(),
            x_len,
            y_len,
        }
    }
}

impl ArrayMatrix<f32> {
    pub fn max_reduce(&self) -> Self {
        assert!(self.x_len.is_multiple_of(2));
        assert!(self.y_len.is_multiple_of(2));
        let mut new = Self::new(self.x_len / 2, self.y_len / 2);
        for x in 0..new.x_len {
            for y in 0..new.y_len {
                let all = [
                    self.get(x * 2, y * 2),
                    self.get(x * 2 + 1, y * 2),
                    self.get(x * 2, y * 2 + 1),
                    self.get(x * 2 + 1, y * 2 + 1),
                ];
                let max = all.into_iter().reduce(f32::max).unwrap();
                new.set(x, y, max);
            }
        }
        new
    }
}

impl ArrayMatrix<f32> {
    pub fn save_as_image(&self, white_value: f32, path: &str) {
        self.as_image(white_value).save(path);
    }

    pub fn as_image(&self, white_value: f32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        image::RgbImage::from_fn(self.x_len as u32, self.y_len as u32, |x, y| {
            let value = self.get(x as usize, y as usize);
            if value >= white_value {
                Rgb([255, 0, 0])
            } else {
                let value_u8 =
                    ((value / white_value).clamp(0.0, 1.0) * 255.0) as u8;
                Rgb([value_u8, value_u8, value_u8])
            }
        })
    }
}

impl<T: Copy + Default> Matrix for ArrayMatrix<T> {
    type Item = T;

    fn x_len(&self) -> usize {
        self.x_len
    }

    fn y_len(&self) -> usize {
        self.y_len
    }

    fn len(&self) -> usize {
        self.store.len()
    }

    fn get(&self, x_index: usize, y_index: usize) -> T {
        debug_assert!(x_index < self.x_len, "{x_index} >= {}", self.x_len);
        debug_assert!(y_index < self.y_len, "{y_index} >= {}", self.y_len);
        // if x_index >= self.x_len || y_index >= self.y_len {
        //     return T::default();
        // }
        self.store[y_index * self.x_len + x_index]
    }

    fn set(&mut self, x_index: usize, y_index: usize, item: T) {
        debug_assert!(x_index < self.x_len, "{x_index} >= {}", self.x_len);
        debug_assert!(y_index < self.y_len, "{y_index} >= {}", self.y_len);
        self.store[y_index * self.x_len + x_index] = item;
    }
}

pub fn isize_indices_in_matrix_bounds(
    matrix: &impl Matrix,
    x_index: isize,
    y_index: isize,
) -> bool {
    x_index >= 0
        && y_index >= 0
        && (x_index as usize) < matrix.x_len()
        && (y_index as usize) < matrix.y_len()
}
