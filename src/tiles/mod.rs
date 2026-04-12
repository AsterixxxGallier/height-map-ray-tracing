use crate::map::Map;
use crate::tiles::download::download_tiles;
use crate::transform::PixelSpacePositionAcrossTiles;
use image::{ImageBuffer, Rgb};
use indicatif::ProgressIterator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use crate::tile::Tile;

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

pub struct Tiles {
    tiles: HashMap<TileCoordinates, Tile>,
}

fn tile_filename(coordinates: TileCoordinates) -> String {
    let tile_id = format_args!("{:0>4}_{:0>4}", coordinates.x, coordinates.y);
    format!("LHD_FXX_{tile_id}_MNS_O_0M50_LAMB93_IGN69.tif")
}

impl Tiles {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
        }
    }

    pub fn load_from_directory(&mut self, region: TileRegion, directory: impl AsRef<Path>) {
        for coordinates in region
            .coordinates()
            .progress_count(region.coordinates().count() as u64)
        {
            let filename = tile_filename(coordinates);
            let path = directory.as_ref().join(filename);
            let file = match File::open(path) {
                Ok(file) => file,
                Err(error) => panic!(
                    "could not open file for tile coordinates x={} y={}: {error}",
                    coordinates.x, coordinates.y,
                ),
            };

            let tile = Map::<f32>::load_from_tiff(2000, 2000, file);
            self.tiles.insert(coordinates, Tile::new(tile));
        }
    }

    pub fn download_and_load_from_directory(
        &mut self,
        region: TileRegion,
        directory: impl AsRef<Path>,
    ) {
        download_tiles(&directory, region);
        self.load_from_directory(region, directory);
    }

    pub fn tile(&self, coordinates: TileCoordinates) -> Option<&Tile> {
        self.tiles.get(&coordinates)
    }

    pub fn as_image(&self, region: TileRegion, white_value: f32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let width = ((region.x_max - region.x_min + 1) * 2000) as u32;
        let height = ((region.y_max - region.y_min + 1) * 2000) as u32;
        image::RgbImage::from_fn(width, height, |x, y| {
            let y = height - 1 - y;
            let position = PixelSpacePositionAcrossTiles {
                x: (region.x_min * 2000 + x as i32) as f64,
                y: (region.y_min * 2000 + y as i32) as f64,
            };
            let (tile_coordinates, position_in_tile) = position.split();
            if let Some(tile) = self.tile(tile_coordinates) {
                let value = tile.map().get(position_in_tile.x as usize, position_in_tile.y as usize);
                if value >= white_value {
                    Rgb([255, 0, 0])
                } else {
                    let value_u8 = ((value / white_value).clamp(0.0, 1.0) * 255.0) as u8;
                    Rgb([value_u8, value_u8, value_u8])
                }
            } else {
                Rgb([255, 0, 255])
            }
        })
    }

    pub fn as_debug_image(
        &self,
        region: TileRegion,
        white_value: f32,
    ) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let x_tile_count = region.x_max - region.x_min + 1;
        let y_tile_count = region.y_max - region.y_min + 1;
        let width = (x_tile_count * 2000) as u32;
        let height = (y_tile_count * 2000) as u32;
        image::RgbImage::from_fn(width, height, |x, y| {
            let y = height - 1 - y;
            let position = PixelSpacePositionAcrossTiles {
                x: (region.x_min * 2000 + x as i32) as f64,
                y: (region.y_min * 2000 + y as i32) as f64,
            };
            let (tile_coordinates, position_in_tile) = position.split();
            if x % 200 < 40 && y % 200 < 40 {
                // Rgb([200, (x * 255 / width) as u8, (y * 255 / height) as u8])
                Rgb([
                    200,
                    ((tile_coordinates.x - region.x_min) * 255 / x_tile_count) as u8,
                    ((tile_coordinates.y - region.y_min) * 255 / y_tile_count) as u8,
                ])
            } else {
                if let Some(tile) = self.tile(tile_coordinates) {
                    let value = tile.map().get(position_in_tile.x as usize, position_in_tile.y as usize);
                    if value >= white_value {
                        Rgb([255, 0, 0])
                    } else {
                        let value_u8 = ((value / white_value).clamp(0.0, 1.0) * 255.0) as u8;
                        Rgb([value_u8, value_u8, value_u8])
                    }
                } else {
                    Rgb([255, 0, 255])
                }
            }
        })
    }
}
