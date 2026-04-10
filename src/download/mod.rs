use std::fs::File;
use std::io;
use std::path::Path;
use reqwest::blocking::Client;

struct TileBounds {
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
}

impl TileBounds {
    fn new(x: i32, y: i32) -> Self {
        Self {
            x_min: (x * 1000) as f32 - 0.25,
            x_max: ((x + 1) * 1000) as f32 - 0.25,
            y_min: ((y - 1) * 1000) as f32 + 0.25,
            y_max: (y * 1000) as f32 + 0.25,
        }
    }
}

fn generate_url_and_filename(x: i32, y: i32) -> (String, String) {
    let bounds = TileBounds::new(x, y);
    let tile_id = format_args!("{x:0>4}_{y:0>4}");
    let filename = format!("LHD_FXX_{tile_id}_MNS_O_0M50_LAMB93_IGN69.tif");
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

pub struct Region {
    pub x_min: i32,
    pub x_max: i32,
    pub y_min: i32,
    pub y_max: i32,
}

pub fn download_tile(directory: &str, x: i32, y: i32) {
    download_tile_with_client(&mut Client::new(), directory, x, y);
}

pub fn download_tile_with_client(client: &Client, directory: &str, x: i32, y: i32) {
    let (url, filename) = generate_url_and_filename(x, y);
    let path = Path::new(directory).join(filename);
    match File::create_new(path) {
        Ok(mut file) => {
            let mut resp = client.get(url).send().expect("request failed");
            io::copy(&mut resp, &mut file).expect("failed to copy content");
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return,
        Err(error) => panic!("failed to create file: {error}"),
    }
}

pub fn download_tiles(directory: &str, region: Region) {
    let client = Client::new();
    for x in region.x_min..=region.x_max {
        for y in region.y_min..=region.y_max {
            download_tile_with_client(&client, directory, x, y);
        }
    }
}
