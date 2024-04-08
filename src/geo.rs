use geojson::{Feature, FeatureCollection, GeoJson, Geometry, LineStringType, Value};
use polyline;

use crate::strava;

pub fn decode_all(activities: Vec<strava::ActivitiesResponse>) -> GeoJson {
    let mut features: Vec<Feature> = Vec::with_capacity(activities.len());
    for activity in activities {
        let polyline = activity
            .map
            .polyline // TODO: get all activities never has polyline
            .as_ref()
            .or(activity.map.summary_polyline.as_ref());

        let geometry: Option<Geometry> = polyline.map(|poly| {
            let line_string = polyline::decode_polyline(poly, 5).unwrap();
            let ls: LineStringType = line_string.into_iter().map(|c| vec![c.x, c.y]).collect();
            Geometry::new(Value::LineString(ls))
        });

        let properties = activity.to_json_object();

        let feat = Feature {
            bbox: None,
            geometry,
            id: None,
            properties,
            foreign_members: None,
        };
        features.push(feat);
    }
    let fc = FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };
    fc.into()
}
