use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use polyline;

use crate::strava;

pub fn decode_all(activities: Vec<strava::ActivitiesResponse>) -> GeoJson {
    let empty_poly = "".to_string();
    let mut feats: Vec<Feature> = Vec::with_capacity(activities.len());
    for activity in activities {
        let poly = activity
            .map
            .polyline
            .as_ref()
            .or(activity.map.summary_polyline.as_ref())
            .unwrap_or(&empty_poly);
        let line_string = polyline::decode_polyline(poly, 5).unwrap();
        let ls: Vec<Vec<f64>> = line_string.into_iter().map(|c| vec![c.x, c.y]).collect();

        let geom = Geometry::new(Value::LineString(ls));
        let feat = Feature {
            bbox: None,
            geometry: Some(geom),
            id: None,
            properties: None,
            foreign_members: None,
        };
        feats.push(feat);
    }
    let fc = FeatureCollection {
        bbox: None,
        features: feats,
        foreign_members: None,
    };
    fc.into()
}
