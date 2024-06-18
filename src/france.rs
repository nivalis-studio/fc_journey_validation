use geo::Geometry;
use once_cell::sync::Lazy;

pub static FRANCE: Lazy<Geometry<f64>> = Lazy::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/france_geometry.json"
    )))
    .expect("Failed to deserialize Geometry")
});
