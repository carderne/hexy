use geo::LineString;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, JsonObject, JsonValue, Value};
use polyline;
use serde::Serialize;

use crate::strava::ActivityResponse;

#[derive(Serialize)]
pub struct Activity {
    pub id: i64,
    pub name: String,
    pub distance: f64,
    pub moving_time: i64,
    pub elapsed_time: i64,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub kudos_count: i32,
    pub average_speed: f64,
    pub sport_type: String,
    pub linestring: Option<LineString>,
}

impl Activity {
    pub fn from_response(obj: ActivityResponse) -> Activity {
        let poly = obj
            .map
            .polyline // TODO: AllActivities endpoint never has `polyline`
            .as_ref()
            .or(obj.map.summary_polyline.as_ref());
        let linestring = poly.map(|poly| polyline::decode_polyline(poly, 5).unwrap());
        Activity {
            id: obj.id,
            name: obj.name,
            distance: obj.distance,
            moving_time: obj.moving_time,
            elapsed_time: obj.elapsed_time,
            start_date: obj.start_date,
            kudos_count: obj.kudos_count,
            average_speed: obj.average_speed,
            sport_type: obj.sport_type,
            linestring,
        }
    }
    pub fn to_properties(&self) -> Option<JsonObject> {
        let mut value = serde_json::to_value(self).unwrap();
        if let JsonValue::Object(ref mut obj) = value {
            obj.remove("linestring");
            Some(obj.clone())
        } else {
            None
        }
    }
}

pub fn decode_all(activities: Vec<ActivityResponse>) -> Vec<Activity> {
    let mut acts: Vec<Activity> = Vec::with_capacity(activities.len());
    for activity in activities {
        let activity = Activity::from_response(activity);
        acts.push(activity);
    }
    acts
}

pub fn to_geojson(activities: Vec<Activity>) -> GeoJson {
    let mut features: Vec<Feature> = Vec::with_capacity(activities.len());
    for activity in activities {
        let properties = activity.to_properties();
        let geometry: Option<Geometry> = activity.linestring.map(|ls| {
            let ls = ls.into_iter().map(|c| vec![c.x, c.y]).collect();
            Geometry::new(Value::LineString(ls))
        });
        let feat = Feature {
            geometry,
            properties,
            bbox: None,
            id: None,
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
