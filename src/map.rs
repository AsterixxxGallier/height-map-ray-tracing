use image::{ImageBuffer, Rgb};
use rand::distr::Distribution;
use rand::Rng;

pub trait Map {
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
pub struct ArrayMap<T> {
    store: Box<[T]>,
    x_len: usize,
    y_len: usize,
}

impl<T: Copy + Default> ArrayMap<T> {
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

impl ArrayMap<f32> {
    pub fn save_as_image(&self, white_value: f32, path: &str) {
        self.as_image(white_value).save(path).unwrap();
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

impl<T: Copy + Default> Map for ArrayMap<T> {
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
        self.store[y_index * self.x_len + x_index]
    }

    fn set(&mut self, x_index: usize, y_index: usize, item: T) {
        debug_assert!(x_index < self.x_len, "{x_index} >= {}", self.x_len);
        debug_assert!(y_index < self.y_len, "{y_index} >= {}", self.y_len);
        self.store[y_index * self.x_len + x_index] = item;
    }
}
