use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub mod download;
pub mod error;
pub mod gmaps;
pub mod http;
pub mod location;

pub type W3W = String;

#[derive(Clone, Deserialize, Serialize)]
pub struct GeoCoords {
    latitude: f64,
    longitude: f64
}

impl GeoCoords {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self { latitude, longitude }
    }

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }
}

/// Simple map format which uses What3Words locations to mark out locations, corners, etc.
#[derive(Deserialize, Debug)]
pub struct Map {
    id: String,
    title: String,
    from: W3W,
    to: W3W,
    locations: Vec<MapLocation>,
}

impl Map {
    pub fn maps_from_dir(map_dir: &Path) -> Result<Vec<Map>> {
        let files_in_dir = fs::read_dir(map_dir).with_context(|| format!("loading map dir '{}'", map_dir.display()))?.flatten().map(|p| p.path());
        let map_files: Vec<_> = files_in_dir
            .into_iter()
            .filter(|p| p.to_string_lossy().ends_with(".yaml"))
            .collect();

        let mut maps = vec![];
        for file in map_files {
            let mut maps_in_file = Self::maps_from_file(&file).context(format!(
                "loading map file in '{}'",
                map_dir.to_string_lossy()
            ))?;
            maps.append(&mut maps_in_file);
        }
        Ok(maps)
    }

    pub fn maps_from_file(map_file: &Path) -> Result<Vec<Map>> {
        let map_file_yaml = fs::read_to_string(map_file)
            .context(format!("loading map file '{}'", map_file.to_string_lossy()))?;

        Self::maps_from_str(&map_file_yaml)
    }

    pub fn maps_from_str(map_file_yaml: &str) -> Result<Vec<Map>> {
        let mut maps = vec![];
        for document in serde_yaml::Deserializer::from_str(map_file_yaml) {
            let map = Map::deserialize(document)?;
            maps.push(map);
        }
        Ok(maps)
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn locations(&self) -> impl Iterator<Item = &MapLocation> {
        self.locations.iter()
    }
}

#[derive(Deserialize, Debug)]
pub struct MapLocation {
    id: String,
    title: String,
    pt: W3W,
    description: Option<String>,
}

impl MapLocation {
    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    pub fn pt(&self) -> W3W {
        self.pt.clone()
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }
}
