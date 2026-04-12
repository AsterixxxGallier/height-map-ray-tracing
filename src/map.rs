use std::fs::File;
use image::{ImageBuffer, Rgb};
use rand::distr::Distribution;
use rand::Rng;
use tiff::decoder::{Decoder, DecodingResult};

#[derive(Debug)]
pub struct Map<T> {
    store: Box<[T]>,
    x_len: usize,
    y_len: usize,
}

impl<T: Copy + Default> Map<T> {
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

    pub fn from_fn(
        x_len: usize,
        y_len: usize,
        f: impl Fn(usize, usize) -> T,
    ) -> Self {
        Self {
            store: (0..x_len * y_len).map(|index| {
                let x_index = index % x_len;
                let y_index = y_len - 1 - index / x_len;
                f(x_index, y_index)
            }).collect(),
            x_len,
            y_len,
        }
    }

    pub fn x_len(&self) -> usize {
        self.x_len
    }

    pub fn y_len(&self) -> usize {
        self.y_len
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn get(&self, x_index: usize, y_index: usize) -> T {
        debug_assert!(x_index < self.x_len, "{x_index} >= {}", self.x_len);
        debug_assert!(y_index < self.y_len, "{y_index} >= {}", self.y_len);
        self.store[(self.y_len - 1 - y_index) * self.x_len + x_index]
    }

    pub fn set(&mut self, x_index: usize, y_index: usize, item: T) {
        debug_assert!(x_index < self.x_len, "{x_index} >= {}", self.x_len);
        debug_assert!(y_index < self.y_len, "{y_index} >= {}", self.y_len);
        self.store[(self.y_len - 1 - y_index) * self.x_len + x_index] = item;
    }
}

impl Map<f32> {
    pub fn load_from_tiff(x_len: usize, y_len: usize, file: File) -> Self {
        let reader = std::io::BufReader::new(file);
        let mut decoder = Decoder::new(reader).unwrap();
        let mut data = DecodingResult::F32(vec![]);

        _ = decoder.read_image_to_buffer(&mut data).unwrap();

        let DecodingResult::F32(data) = data else {
            panic!()
        };

        Self::from_vec(x_len, y_len, data)
    }

    pub fn save_as_image(&self, white_value: f32, path: &str) {
        self.as_image(white_value).save(path).unwrap();
    }

    pub fn as_image(&self, white_value: f32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        image::RgbImage::from_fn(self.x_len as u32, self.y_len as u32, |x, y| {
            let value = self.get(x as usize, self.y_len - 1 - y as usize);
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
