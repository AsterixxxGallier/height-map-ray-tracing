use std::fs::File;
use std::io;
use std::path::Path;
use reqwest::blocking::Client;
use crate::tiles::{tile_filename, TileCoordinates, TileRegion};

struct TileBounds {
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
}

impl TileBounds {
    fn new(coordinates: TileCoordinates) -> Self {
        Self {
            x_min: (coordinates.x * 1000) as f32 - 0.25,
            x_max: ((coordinates.x + 1) * 1000) as f32 - 0.25,
            y_min: ((coordinates.y - 1) * 1000) as f32 + 0.25,
            y_max: (coordinates.y * 1000) as f32 + 0.25,
        }
    }
}

fn tile_url_and_filename(coordinates: TileCoordinates) -> (String, String) {
    let bounds = TileBounds::new(coordinates);
    let filename = tile_filename(coordinates);
    let url = format!(
        "https://data.geopf.fr/wms-r\
        ?SERVICE=WMS&VERSION=1.3.0&EXCEPTIONS=text/xml&REQUEST=GetMap\
        &LAYERS=IGNF_LIDAR-HD_MNS_ELEVATION.ELEVATIONGRIDCOVERAGE.LAMB93\
        &FORMAT=image/geotiff&STYLES=&CRS=EPSG:2154\
        &BBOX={},{},{},{}\
        &WIDTH=2000&HEIGHT=2000\
        &FILENAME={filename}",
        bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max,
    );
    (url, filename)
}

#[inline(never)]
#[cold]
pub fn download_tile(directory: impl AsRef<Path>, coordinates: TileCoordinates) {
    download_tile_with_client(&mut Client::new(), directory, coordinates);
}

pub fn download_tile_with_client(client: &Client, directory: impl AsRef<Path>, coordinates: TileCoordinates) {
    let (url, filename) = tile_url_and_filename(coordinates);
    let path = directory.as_ref().join(filename);
    match File::create_new(path) {
        Ok(mut file) => {
            let mut resp = client.get(url).send().expect("request failed");
            io::copy(&mut resp, &mut file).expect("failed to copy content");
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return,
        Err(error) => panic!("failed to create file: {error}"),
    }
}

pub fn download_tiles(directory: impl AsRef<Path>, region: TileRegion) {
    let client = Client::new();
    for coordinates in region.coordinates() {
        download_tile_with_client(&client, directory.as_ref(), coordinates);
    }
}
