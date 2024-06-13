use anyhow::Result;
use geo::LineString;
use geojson::{Feature, FeatureCollection, Geometry};
use std::fs::File;
use std::io::Write;

pub fn traces_to_geojson(t1: &LineString, t2: &LineString) -> Result<()> {
    let mut stroke1 = geojson::JsonObject::new();

    stroke1.insert("stroke".to_string(), geojson::JsonValue::from("#00a3d7"));
    stroke1.insert("stroke-width".to_string(), geojson::JsonValue::from("2"));
    stroke1.insert("stroke-opacity".to_string(), geojson::JsonValue::from("1"));

    let mut stroke2 = geojson::JsonObject::new();

    stroke2.insert("stroke".to_string(), geojson::JsonValue::from("#ff6251"));
    stroke2.insert("stroke-width".to_string(), geojson::JsonValue::from("2"));
    stroke2.insert("stroke-opacity".to_string(), geojson::JsonValue::from("1"));

    let mut file = File::create("traces.geojson")?;
    file.write_all(
        FeatureCollection {
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
                    properties: Some(stroke1),
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
                    properties: Some(stroke2),
                    foreign_members: None,
                },
            ],
            foreign_members: None,
        }
        .to_string()
        .as_bytes(),
    )?;

    Ok(())
}
