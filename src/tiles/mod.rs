use crate::map::Map;
use crate::tiles::download::download_tiles;
use std::collections::HashMap;
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
    pub fn coordinates(&self) -> impl Iterator<Item = TileCoordinates> {
        (self.x_min..=self.x_max)
            .flat_map(|x| (self.y_min..=self.y_max).map(move |y| TileCoordinates { x, y }))
    }
}

pub struct Tiles {
    tiles: HashMap<TileCoordinates, Map<f32>>,
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

    pub fn load_from_directory(&mut self, region: TileRegion, directory: &str) {
        for coordinates in region.coordinates() {
            let filename = tile_filename(coordinates);
            let path = Path::new(directory).join(filename);
            let file = match File::open(path) {
                Ok(file) => file,
                Err(error) => panic!(
                    "could not open file for tile coordinates x={} y={}: {error}",
                    coordinates.x, coordinates.y,
                ),
            };

            let tile = Map::<f32>::load_from_tiff(2000, 2000, file);
            self.tiles.insert(coordinates, tile);
        }
    }

    pub fn download_and_load_from_directory(&mut self, region: TileRegion, directory: &str) {
        download_tiles(directory, region);
        self.load_from_directory(region, directory);
    }

    pub fn tile(&self, coordinates: TileCoordinates) -> Option<&Map<f32>> {
        self.tiles.get(&coordinates)
    }
}
