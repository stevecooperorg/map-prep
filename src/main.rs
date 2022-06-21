use anyhow::{Context, Result};
use clap::Parser;
use http::Uri;
use map_prep::gmaps::GoogleMaps;
use map_prep::location::LocationCache;
use map_prep::{download, Map};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// the directory that contains your map yaml files
    #[clap(short, long)]
    map_dir: PathBuf,
    /// the file that contains the location cache. You can choose to source control this file, or not.
    #[clap(short, long)]
    location_file: PathBuf,
    /// the directory to download maps to - this is a cache directory and should not be checked into source control.
    #[clap(short, long)]
    download_dir: PathBuf,
    /// the directory where the final maps will be written to, ready to be used.
    #[clap(short, long)]
    output_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!();
    println!("map-prep");
    println!();
    println!("  Reading maps from {:?}", args.map_dir);
    println!("  Location file at {:?}", args.location_file);
    println!("  Downloading files to {:?}", args.download_dir);
    println!("  Outputting final files to {:?}", args.output_dir);
    println!();

    let maps = Map::maps_from_dir(&args.map_dir)?;

    println!("Loaded {} maps", maps.len());
    println!();

    let location_cache_dir = args
        .location_file
        .parent()
        .expect("location file has no parent dir!");

    if !location_cache_dir.exists() {
        // make sure we've got a directory to download into
        fs::create_dir_all(&location_cache_dir)?;
        println!(
            "Created missing location cache dir: '{}'",
            location_cache_dir.to_string_lossy()
        );
        println!();
    }

    let location_cache = LocationCache::build(&maps, &args.location_file).await?;

    let google_maps = GoogleMaps::init_from_env()?;

    if !args.download_dir.exists() {
        // make sure we've got a directory to download into
        fs::create_dir_all(&args.download_dir)?;
    }

    if !args.output_dir.exists() {
        // make sure we've got a directory to copy the final output to
        fs::create_dir_all(&args.output_dir)?;
    }

    let downloader = download::CachedDownloader::new(args.download_dir.clone(), "png");

    for map in &maps {
        // create a description of the map (coordinates, markers, etc
        let prepared = google_maps.prepare(map, &location_cache)?;
        // this is the static URL for the map
        let uri: Uri = prepared.url().parse()?;
        // download a cached version to the hashed name, if it's not there;
        let saved_path = downloader.download_if_missing(&map.id(), &uri).await?;
        // copy the map to where you actually want to use it - eg, in a website, markdown document, etc.

        let output_file_name = format!("{}.{}", map.id(), "png");
        let output_path = args.output_dir.join(output_file_name);
        fs::copy(&saved_path, &output_path).context("copying downloaded file into place")?;

        println!("map: {} as {} ", map.id(), output_path.to_string_lossy());
    }

    println!("Done");

    Ok(())
}
