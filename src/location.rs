use crate::{GeoCoords, Map, W3W};
use anyhow::{Context, Result};
use gmaps_static::Location;
use http::Uri;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// https://developer.what3words.com/public-api/docs#overview
const W3W_CONVERT_URI: &str =
    "https://api.what3words.com/v3/convert-to-coordinates?words={{WORDS}}&key={{API-KEY}}";
const W3W_API_ENV_VAR: &str = "WHAT3WORDS_API_KEY";

pub struct LocationCache {
    cache: HashMap<W3W, GeoCoords>,
    w3w_client: What3WordsClient,
}

impl LocationCache {
    fn new(cache: HashMap<W3W, GeoCoords>, w3w_client: What3WordsClient) -> Self {
        Self { cache, w3w_client }
    }

    async fn insert(&mut self, pt: &W3W) -> Result<()> {
        if !self.cache.contains_key(pt) {
            println!("looking up missing lcation: {}", pt);
            let coords = self.w3w_client.convert_words(pt).await?;
            self.cache.insert(pt.clone(), coords);
        }

        Ok(())
    }

    pub fn get(&self, pt: &W3W) -> Option<GeoCoords> {
        self.cache.get(pt).cloned()
    }

    pub fn get_gmaps_location(&self, pt: &W3W) -> Option<Location> {
        self.get(pt).map(|loc| Location::LatLng(loc.latitude, loc.longitude))
    }

    pub fn deserialize(yaml: &str, w3w_client: What3WordsClient) -> Result<Self> {
        let cache = serde_yaml::from_str(yaml).context("could not deserialize location cache")?;
        Ok(Self::new(cache, w3w_client))
    }

    fn serialize(&self) -> Result<String> {
        let yaml = serde_yaml::to_string(&self.cache)?;
        Ok(yaml)
    }

    pub async fn build(maps: &[Map], location_file: &Path) -> Result<LocationCache> {
        let w3w_client = What3WordsClient::init_from_env()?;

        let mut location_cache = if location_file.exists() {
            let location_file_yaml = fs::read_to_string(&location_file).context(format!(
                "reading location file - if this fails try deleting '{}'",
                location_file.to_string_lossy()
            ))?;
            LocationCache::deserialize(&location_file_yaml, w3w_client)?
        } else {
            LocationCache::new(HashMap::new(), w3w_client)
        };

        for map in maps {
            location_cache.insert(&map.from).await?;
            location_cache.insert(&map.to).await?;
            for loc in &map.locations {
                location_cache.insert(&loc.pt).await?;
            }
        }

        println!("All Locations:");
        for (pt, coords) in &location_cache.cache {
            println!("{} -> {},{}", pt, coords.latitude, coords.longitude);
        }

        // serialise and save location cache
        let location_cache_yaml = location_cache.serialize()?;
        fs::write(location_file, location_cache_yaml)?;

        Ok(location_cache)
    }
}

#[derive(Deserialize, Debug)]
struct ConvertResponse {
    coordinates: ConvertLatLongResponse,
}

#[derive(Deserialize, Debug)]
struct ConvertLatLongResponse {
    lng: f64,
    lat: f64,
}

impl ConvertResponse {
    fn to_geo_coords(&self) -> GeoCoords  {
        GeoCoords::new(self.coordinates.lat, self.coordinates.lng)
    }
}

pub struct What3WordsClient {
    api_key: String,
}

impl What3WordsClient {
    pub fn init_from_env() -> Result<Self> {
        let api_key = std::env::var(W3W_API_ENV_VAR)
            .context("initializing W3W client from W3W_API_ENV_VAR env var")?;

        Ok(Self { api_key })
    }

    pub async fn convert_words(&self, words: &W3W) -> Result<GeoCoords> {
        let uri = W3W_CONVERT_URI
            .replace("{{WORDS}}", words)
            .replace("{{API-KEY}}", &self.api_key);

        let uri: Uri = Uri::from_str(&uri)?;
        let resp: ConvertResponse = crate::http::fetch_json(&uri).await?;
        Ok(resp.to_geo_coords())
    }
}
