use crate::map::Map;
use crate::tile::Tile;
use crate::tiles::download::download_tile;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::fs::File;
use std::path::Path;

pub mod download;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct TileCoordinates {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct TileRegion {
    pub x_min: i32,
    pub x_max: i32,
    pub y_min: i32,
    pub y_max: i32,
}

impl TileRegion {
    pub fn area(&self) -> usize {
        ((self.x_max - self.x_min + 1) * (self.y_max - self.y_min + 1)) as usize
    }

    pub fn coordinates(&self) -> impl Iterator<Item = TileCoordinates> {
        (self.x_min..=self.x_max)
            .flat_map(|x| (self.y_min..=self.y_max).map(move |y| TileCoordinates { x, y }))
    }

    pub fn par_coordinates(&self) -> impl ParallelIterator<Item = TileCoordinates> {
        (self.x_min..=self.x_max).into_par_iter().flat_map(|x| {
            (self.y_min..=self.y_max)
                .into_par_iter()
                .map(move |y| TileCoordinates { x, y })
        })
    }
}

fn tile_filename(coordinates: TileCoordinates) -> String {
    let tile_id = format_args!("{:0>4}_{:0>4}", coordinates.x, coordinates.y);
    format!("LHD_FXX_{tile_id}_MNS_O_0M50_LAMB93_IGN69.tif")
}

pub fn load_tile(directory: impl AsRef<Path>, coordinates: TileCoordinates) -> Tile {
    let filename = tile_filename(coordinates);
    let path = directory.as_ref().join(filename);
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => panic!(
            "could not open file for tile coordinates x={} y={}: {error}",
            coordinates.x, coordinates.y,
        ),
    };

    let map = Map::<f32>::load_from_tiff(2000, 2000, file);
    Tile::new(map)
}

pub fn download_and_load_tile(directory: impl AsRef<Path>, coordinates: TileCoordinates) -> Tile {
    download_tile(&directory, coordinates);
    load_tile(directory, coordinates)
}
