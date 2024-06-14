use anyhow::Result;
use geo::LineString;
use geojson::{Feature, FeatureCollection, Geometry, JsonObject, JsonValue};
use std::collections::HashMap;

pub fn traces_to_geojson(t1: &LineString, t2: &LineString) -> Result<()> {
    let feature_collection = FeatureCollection {
        bbox: None,
        features: vec![
            Feature {
                bbox: None,
                geometry: Some(Geometry {
                    value: geojson::Value::from(&t1.clone()),
                    bbox: None,
                    foreign_members: None,
                }),
                id: None,
                properties: create_properties("#00a3d7", "2", "1"),
                foreign_members: None,
            },
            Feature {
                bbox: None,
                geometry: Some(Geometry {
                    value: geojson::Value::from(&t2.clone()),
                    bbox: None,
                    foreign_members: None,
                }),
                id: None,
                properties: create_properties("#ff6251", "2", "1"),
                foreign_members: None,
            },
        ],
        foreign_members: None,
    }
    .to_string();

    let uri_data = urlencoding::encode(&feature_collection);

    let url = format!("http://geojson.io/#data=data:application/json,{}", uri_data);

    open::that(url)?;

    Ok(())
}

fn create_properties(color: &str, width: &str, opacity: &str) -> Option<JsonObject> {
    let mut properties = JsonObject::new();
    let properties_: HashMap<String, JsonValue> = [
        ("stroke".to_string(), JsonValue::from(color)),
        ("stroke-width".to_string(), JsonValue::from(width)),
        ("stroke-opacity".to_string(), JsonValue::from(opacity)),
    ]
    .iter()
    .cloned()
    .collect();

    properties.extend(properties_);

    Some(properties)
}
