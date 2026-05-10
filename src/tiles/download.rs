use std::path::Path;
use std::sync::Arc;
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::time::{interval, Duration};
use tokio::io::AsyncWriteExt;
use indicatif::{ProgressBar, ProgressStyle};
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
pub async fn download_tile_async(client: &Client, directory: impl AsRef<Path>, coordinates: TileCoordinates) {
    let (url, filename) = tile_url_and_filename(coordinates);
    let path = directory.as_ref().join(filename);

    // Use Tokio's async filesystem operations
    let file_result = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await;

    let mut file = match file_result {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return,
        Err(error) => panic!("failed to create file: {error}"),
    };

    // Execute the async network request
    let mut response = client.get(&url).send().await.expect("request failed");

    // Stream the response body directly to the file without loading it entirely into RAM
    while let Some(chunk) = response.chunk().await.expect("failed to read chunk") {
        file.write_all(&chunk).await.expect("failed to write to file");
    }
}

pub async fn download_tiles(directory: impl AsRef<Path>, region: TileRegion) {
    let client = Client::new();

    // 1. Concurrency Control: Cap at 50 simultaneous downloads.
    // You can tune this based on your bandwidth.
    let max_concurrent_downloads = 50;
    let semaphore = Arc::new(Semaphore::new(max_concurrent_downloads));

    // 2. Rate Limiting: 10 requests per second = 1 tick every 100ms.
    let mut rate_limit = interval(Duration::from_millis(100));

    // Progress bar setup
    let total_tiles = region.area() as u64;
    let pb = ProgressBar::new(total_tiles);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-"));

    let dir = directory.as_ref().to_path_buf();
    let mut tasks = Vec::new();

    for coordinates in region.coordinates() {
        // Wait until 100ms has passed since the last tick
        rate_limit.tick().await;

        // Acquire a permit. If 50 downloads are actively running, this will pause
        // the loop until one finishes, preventing runaway memory usage.
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        // Clone references for the async task
        let client = client.clone();
        let dir = dir.clone();
        let pb = pb.clone();

        // Spawn a lightweight Tokio task
        let task = tokio::spawn(async move {
            // The permit is moved into this closure and automatically dropped
            // when the download completes, freeing up a slot.
            let _permit = permit;

            download_tile_async(&client, &dir, coordinates).await;
            pb.inc(1);
        });

        tasks.push(task);
    }

    // Await the completion of all spawned tasks
    for task in tasks {
        let _ = task.await;
    }

    pb.finish_with_message("All tiles downloaded!");
}