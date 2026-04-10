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

    pub fn from_vec(x_len: usize, y_len: usize, vec: Vec<T>) -> Self {
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
                let value_u8 = ((value / white_value).clamp(0.0, 1.0) * 255.0) as u8;
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

#[derive(Debug)]
pub struct ChunkedArrayMap<T> {
    store: Box<[T]>,
    x_len: usize,
    y_len: usize,
    x_chunk_len: usize,
    y_chunk_len: usize,
    chunk_len: usize,
}

impl<T: Copy + Default> ChunkedArrayMap<T> {
    pub fn new(x_len: usize, y_len: usize, x_chunk_len: usize, y_chunk_len: usize) -> Self {
        let store = vec![T::default(); x_len * y_len].into_boxed_slice();
        assert!(x_len.is_multiple_of(x_chunk_len));
        assert!(y_len.is_multiple_of(y_chunk_len));
        Self {
            store,
            x_len,
            y_len,
            x_chunk_len,
            y_chunk_len,
            chunk_len: x_chunk_len * y_chunk_len,
        }
    }

    pub fn from_vec(
        x_len: usize,
        y_len: usize,
        x_chunk_len: usize,
        y_chunk_len: usize,
        vec: Vec<T>,
    ) -> Self {
        let mut this = Self::new(x_len, y_len, x_chunk_len, y_chunk_len);
        let temporary = ArrayMap::from_vec(x_len, y_len, vec);
        for x in 0..x_len {
            for y in 0..y_len {
                this.set(x, y, temporary.get(x, y))
            }
        }
        this
    }
}

impl ChunkedArrayMap<f32> {
    pub fn save_as_image(&self, white_value: f32, path: &str) {
        self.as_image(white_value).save(path).unwrap();
    }

    pub fn as_image(&self, white_value: f32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        image::RgbImage::from_fn(self.x_len as u32, self.y_len as u32, |x, y| {
            let value = self.get(x as usize, y as usize);
            if value >= white_value {
                Rgb([255, 0, 0])
            } else {
                let value_u8 = ((value / white_value).clamp(0.0, 1.0) * 255.0) as u8;
                Rgb([value_u8, value_u8, value_u8])
            }
        })
    }
}

impl<T: Copy + Default> Map for ChunkedArrayMap<T> {
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

        let x_chunk_index = x_index / self.x_chunk_len;
        let y_chunk_index = y_index / self.y_chunk_len;
        let x_index_in_chunk = x_index % self.x_chunk_len;
        let y_index_in_chunk = y_index % self.y_chunk_len;

        self.store[y_chunk_index * self.x_len * self.y_chunk_len
            + x_chunk_index * self.chunk_len
            + y_index_in_chunk * self.x_chunk_len
            + x_index_in_chunk]
    }

    fn set(&mut self, x_index: usize, y_index: usize, item: T) {
        debug_assert!(x_index < self.x_len, "{x_index} >= {}", self.x_len);
        debug_assert!(y_index < self.y_len, "{y_index} >= {}", self.y_len);

        let x_chunk_index = x_index / self.x_chunk_len;
        let y_chunk_index = y_index / self.y_chunk_len;
        let x_index_in_chunk = x_index % self.x_chunk_len;
        let y_index_in_chunk = y_index % self.y_chunk_len;

        self.store[y_chunk_index * self.x_len * self.y_chunk_len
            + x_chunk_index * self.chunk_len
            + y_index_in_chunk * self.x_chunk_len
            + x_index_in_chunk] = item;
    }
}

#[derive(Debug)]
pub struct ConstantChunkedArrayMap<T, const CHUNK_SIZE: usize> {
    store: Box<[T]>,
    x_len: usize,
    y_len: usize,
}

impl<T: Copy + Default, const CHUNK_SIZE: usize> ConstantChunkedArrayMap<T, CHUNK_SIZE> {
    pub fn new(x_len: usize, y_len: usize) -> Self {
        let store = vec![T::default(); x_len * y_len].into_boxed_slice();
        assert!(x_len.is_multiple_of(CHUNK_SIZE));
        assert!(y_len.is_multiple_of(CHUNK_SIZE));
        Self {
            store,
            x_len,
            y_len,
        }
    }

    pub fn from_fn(x_len: usize, y_len: usize, f: impl Fn(usize, usize) -> T) -> Self {
        let mut this = Self::new(x_len, y_len);
        for x in 0..x_len {
            for y in 0..y_len {
                this.set(x, y, f(x, y))
            }
        }
        this
    }
}

impl<const CHUNK_SIZE: usize> ConstantChunkedArrayMap<f32, CHUNK_SIZE> {
    pub fn save_as_image(&self, white_value: f32, path: &str) {
        self.as_image(white_value).save(path).unwrap();
    }

    pub fn as_image(&self, white_value: f32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        image::RgbImage::from_fn(self.x_len as u32, self.y_len as u32, |x, y| {
            let value = self.get(x as usize, y as usize);
            if value >= white_value {
                Rgb([255, 0, 0])
            } else {
                let value_u8 = ((value / white_value).clamp(0.0, 1.0) * 255.0) as u8;
                Rgb([value_u8, value_u8, value_u8])
            }
        })
    }
}

impl<T: Copy + Default, const CHUNK_SIZE: usize> Map for ConstantChunkedArrayMap<T, CHUNK_SIZE> {
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

        let x_chunk_index = x_index / CHUNK_SIZE;
        let y_chunk_index = y_index / CHUNK_SIZE;
        let x_index_in_chunk = x_index % CHUNK_SIZE;
        let y_index_in_chunk = y_index % CHUNK_SIZE;

        self.store[y_chunk_index * self.x_len * CHUNK_SIZE
            + x_chunk_index * CHUNK_SIZE * CHUNK_SIZE
            + y_index_in_chunk * CHUNK_SIZE
            + x_index_in_chunk]
    }

    fn set(&mut self, x_index: usize, y_index: usize, item: T) {
        debug_assert!(x_index < self.x_len, "{x_index} >= {}", self.x_len);
        debug_assert!(y_index < self.y_len, "{y_index} >= {}", self.y_len);

        let x_chunk_index = x_index / CHUNK_SIZE;
        let y_chunk_index = y_index / CHUNK_SIZE;
        let x_index_in_chunk = x_index % CHUNK_SIZE;
        let y_index_in_chunk = y_index % CHUNK_SIZE;

        self.store[y_chunk_index * self.x_len * CHUNK_SIZE
            + x_chunk_index * CHUNK_SIZE * CHUNK_SIZE
            + y_index_in_chunk * CHUNK_SIZE
            + x_index_in_chunk] = item;
    }
}
