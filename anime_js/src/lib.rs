use std::collections::BTreeMap;
use std::sync::Once;

use anime::Anime;
use anyhow::Result;
use geo::{GeometryCollection, LineString};
use geojson::de::deserialize_geometry;
use serde::{Deserialize, Serialize};
use utils::Mercator;
use wasm_bindgen::prelude::*;

static START: Once = Once::new();

/// Takes two GeoJSONs, matches LineStrings, and returns the TODO   optional index of the best matching
/// feature for each target.
#[wasm_bindgen(js_name = matchLineStrings)]
pub fn match_linestrings(
    source_gj: String,
    target_gj: String,
    raw_options: JsValue,
) -> Result<String, JsValue> {
    // Panics shouldn't happen, but if they do, console.log them.
    console_error_panic_hook::set_once();
    START.call_once(|| {
        console_log::init_with_level(log::Level::Info).unwrap();
    });

    let options: Options = serde_wasm_bindgen::from_value(raw_options)?;

    let mut sources: Vec<Input> =
        geojson::de::deserialize_feature_collection_str_to_vec(&source_gj).map_err(err_to_js)?;
    let mut targets: Vec<Input> =
        geojson::de::deserialize_feature_collection_str_to_vec(&target_gj).map_err(err_to_js)?;

    // TODO Expensive clones
    let collection = GeometryCollection::from(
        sources
            .iter()
            .chain(targets.iter())
            .map(|x| x.geometry.clone())
            .collect::<Vec<_>>(),
    );
    let Some(mercator) = Mercator::from(collection) else {
        return Err(JsValue::from_str("empty inputs"));
    };
    for x in sources.iter_mut().chain(targets.iter_mut()) {
        mercator.to_mercator_in_place(&mut x.geometry);
    }

    let mut anime = Anime::load_geometries(
        sources.into_iter().map(|x| x.geometry),
        targets.into_iter().map(|x| x.geometry),
        options.distance_tolerance,
        options.angle_tolerance,
    );
    // TODO map_err after detangling the OnceCell
    anime.find_matches().unwrap();
    let out = transform_matches_map(anime.matches.get().unwrap());
    serde_json::to_string(&out).map_err(err_to_js)
}

#[derive(Deserialize)]
struct Options {
    distance_tolerance: f64,
    angle_tolerance: f64,
}

#[derive(Deserialize)]
struct Input {
    #[serde(deserialize_with = "deserialize_geometry")]
    geometry: LineString,
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}

// TODO Upstream?
fn transform_matches_map(map: &anime::MatchesMap) -> BTreeMap<usize, Vec<MatchCandidate>> {
    map.into_iter()
        .map(|(k, v)| {
            (
                *k as usize,
                v.into_iter()
                    .map(|mc| MatchCandidate {
                        target_index: mc.index as usize,
                        shared_length: mc.shared_len,
                    })
                    .collect(),
            )
        })
        .collect()
}

#[derive(Serialize)]
struct MatchCandidate {
    pub target_index: usize,
    pub shared_length: f64,
}
