use rand::distr::Distribution;
use rand::Rng;

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
}

impl<T: Copy> Matrix for ArrayMatrix<T> {
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
        self.store[y_index * self.x_len + x_index]
    }

    fn set(&mut self, x_index: usize, y_index: usize, item: T) {
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
