use crate::map::Map;
use crate::tiles::download::download_tiles;
use crate::transform::{model_to_pixel, pixel_to_model, Coord, CoordinateTransform};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tiff::decoder::{Decoder, DecodingResult};
use tiff::tags::Tag;

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

            let reader = BufReader::new(file);
            let mut decoder = Decoder::new(reader).unwrap();
            let mut data = DecodingResult::F32(vec![]);

            _ = decoder.read_image_to_buffer(&mut data).unwrap();

            let DecodingResult::F32(data) = data else {
                panic!()
            };

            let pixel_scale_data = decoder
                .find_tag(Tag::ModelPixelScaleTag)
                .unwrap()
                .map(|value| value.into_f64_vec())
                .transpose()
                .unwrap();
            let tie_points_data = decoder
                .find_tag(Tag::ModelTiepointTag)
                .unwrap()
                .map(|value| value.into_f64_vec())
                .transpose()
                .unwrap();
            let model_transformation_data = decoder
                .find_tag(Tag::ModelTransformationTag)
                .unwrap()
                .map(|value| value.into_f64_vec())
                .transpose()
                .unwrap();

            let transform = CoordinateTransform::from_tag_data(
                pixel_scale_data,
                tie_points_data,
                model_transformation_data,
            );

            let transform1 = CoordinateTransform::TiePointAndPixelScale {
                raster_point: Coord { x: 0.0, y: 0.0 },
                model_point: Coord {
                    x: coordinates.x as f64 * 1000.0 - 0.25,
                    y: coordinates.y as f64 * 1000.0,
                },
                pixel_scale: Coord { x: 0.5, y: 0.5 },
            };
            let Coord {
                x: expected_x,
                y: expected_y,
            } = transform.transform_to_model(&Coord { x: 1999.0, y: 1999.0 });
            let (actual_x, actual_y) = pixel_to_model(coordinates, 1999.0, 1999.0);
            assert_eq!(expected_x, actual_x);
            assert_eq!(expected_y, actual_y);
            let (back_coords, back_x, back_y) = model_to_pixel(actual_x, actual_y);
            assert_eq!(back_coords, coordinates);
            assert_eq!(back_x, 1999.0);
            assert_eq!(back_y, 1999.0);
            assert_eq!(transform, transform1);

            let tile = Map::<f32>::from_vec(2000, 2000, data);

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
