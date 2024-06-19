use std::collections::HashMap;

use geo::LineString;
use geojson::{Feature, FeatureCollection, JsonObject, JsonValue};

#[derive(Default)]
pub struct FeatureProperties {
    pub color: String,
    pub width: Option<u8>,
    pub opacity: Option<f32>,
}

impl FeatureProperties {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn color(mut self, color: &str) -> Self {
        self.color = color.to_string();

        self
    }
}

fn create_properties(props: FeatureProperties) -> Option<JsonObject> {
    let mut properties = JsonObject::new();
    let properties_: HashMap<String, JsonValue> = [
        ("stroke".to_string(), JsonValue::from(props.color)),
        (
            "stroke-width".to_string(),
            JsonValue::from(props.width.unwrap_or(2)),
        ),
        (
            "stroke-opacity".to_string(),
            JsonValue::from(props.opacity.unwrap_or(1.0)),
        ),
    ]
    .iter()
    .cloned()
    .collect();

    properties.extend(properties_);

    Some(properties)
}

pub fn visualize<T, I>(features: T)
where
    I: Into<LineString<f64>>,
    T: IntoIterator<Item = (I, FeatureProperties)>,
{
    let mut geojson_features = vec![];

    for (linestring, properties) in features {
        let linestring: LineString = linestring.into();
        let geojson_geometry = geojson::Geometry {
            value: geojson::Value::from(&linestring),
            bbox: None,
            foreign_members: None,
        };
        let feature = Feature {
            bbox: None,
            geometry: Some(geojson_geometry),
            id: None,
            properties: create_properties(properties),
            foreign_members: None,
        };
        geojson_features.push(feature);
    }

    let feature_collection = FeatureCollection {
        bbox: None,
        features: geojson_features,
        foreign_members: None,
    };

    let geojson = feature_collection.to_string();

    let uri_data = urlencoding::encode(&geojson);
    let url = format!("http://geojson.io/#data=data:application/json,{}", uri_data);

    open::that(url).unwrap();
}
