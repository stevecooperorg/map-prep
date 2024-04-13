use crate::location::LocationCache;
use crate::{GeoCoords, Map};
use anyhow::{Context, Result};
use gmaps_static::marker::{Appearence, Label, Marker};
use gmaps_static::{Format, Map as GMap, MapType, SCALE2};

pub struct GoogleMaps {
    api_key: String,
}

impl GoogleMaps {
    pub fn init_from_env() -> Result<GoogleMaps> {
        let api_key = std::env::var("GOOGLE_MAPS_API_KEY")
            .context("initializing Google Static Maps client from GOOGLE_MAPS_API_KEY env var")?;

        Ok(Self { api_key })
    }

    pub fn prepare(&self, map: &Map, location_cache: &LocationCache) -> Result<GMap> {
        const MAX_MAP_EDGE_PX: i32 = 2500;

        let credentials = self.api_key.clone().into();

        let mut locations: Vec<GeoCoords> = vec![
            location_cache
                .get(&map.from)
                .context("getting 'from' from location cache")?,
            location_cache
                .get(&map.to)
                .context("getting 'to' from location cache")?,
        ];

        let from = location_cache
            .get_gmaps_location(&map.from)
            .context("getting 'from' gmaps location from location cache")?;
        let to = location_cache
            .get_gmaps_location(&map.to)
            .context("getting 'to' gmaps location from location cache")?;

        let visible = vec![from.into(), to.into()];

        let mut markers = vec![];
        for pt in &map.locations {
            locations.push(
                location_cache
                    .get(&pt.pt)
                    .context(format!("getting '{}' from location cache", pt.id))?,
            );

            let loc = location_cache.get_gmaps_location(&pt.pt).context(format!(
                "getting '{}' gmaps location from location cache",
                pt.id
            ))?;
            let c: char = pt
                .title
                .chars()
                .next()
                .context("getting first character from title")?;
            let label = Label::new(c).unwrap();
            let style = gmaps_static::marker::Style::new().label(label);
            let appearance = Appearence::Styled(style);
            let marker = Marker::from(loc).appearence(appearance);
            markers.push(marker);
        }

        let (left, top, bottom, right) = {
            let mut left: f64 = f64::MAX;
            let mut right: f64 = f64::MIN;
            let mut top: f64 = f64::MAX;
            let mut bottom: f64 = f64::MIN;
            for loc in &map.locations {
                let loc = location_cache
                    .get(&loc.pt)
                    .context(format!("getting '{}' from location cache", loc.id))?;
                left = left.min(loc.longitude);
                right = right.max(loc.longitude);
                top = top.min(loc.latitude);
                bottom = bottom.max(loc.latitude);
            }
            (left, top, bottom, right)
        };

        let width = right - left;
        let height = bottom - top;
        let ratio = width as f64 / height as f64;
        let (width, height) = if ratio > 1.0f64 {
            // landscape
            (MAX_MAP_EDGE_PX, MAX_MAP_EDGE_PX / ratio as i32)
        } else {
            // portrait
            (MAX_MAP_EDGE_PX / ratio as i32, MAX_MAP_EDGE_PX)
        };

        let size = (width, height).into();

        Ok(GMap::new(credentials, size)
            .format(Format::Png)
            .scale(SCALE2)
            .visible(visible)
            .maptype(MapType::Satellite)
            .markers(markers))
    }
}
