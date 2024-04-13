use std::collections::HashMap;

use dbscan;
use geo::{Centroid, MultiPoint, Point};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};

use crate::models::Activity;
use crate::strava::ActivityResponse;

/// Convert Strava responses to our Activity model, with the only real difference
/// being that polylines are converted to geo::Linestring
pub fn decode_all(activities: Vec<ActivityResponse>) -> Vec<Activity> {
    let mut acts: Vec<Activity> = Vec::with_capacity(activities.len());
    for activity in activities {
        let activity = Activity::from_response(activity);
        acts.push(activity);
    }
    acts
}

/// Convert Activities to GeoJSON with properties
/// Only use this for final web response, as GeoJSON isn't
/// useful for processing
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

/// Get a useful centroid to zoom the user to
/// This function is a bit of a monster, could be probably be made much faster
pub fn get_useful_centroid(activities: &[Activity]) -> Option<Point> {
    // Get all activity centroids
    let centroids: Vec<Vec<f64>> = activities
        .iter()
        .filter_map(|a| a.linestring.clone())
        .filter_map(|l| l.centroid())
        .map(|c| vec![c.x(), c.y()])
        .collect();

    let model: dbscan::Model<f64> = dbscan::Model::new(0.1, 10);
    let classifications = model.run(&centroids);

    // Count number in each group
    let mut counts = HashMap::new();
    for value in classifications.clone().into_iter() {
        if let dbscan::Classification::Core(num) = value {
            *counts.entry(num).or_insert(0) += 1;
        }
    }

    // Find the Core value with the highest count
    // Basicaly, the biggest group
    let group_num = counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(size, _)| size)?;

    // Now get all the centroids that represent that group
    let biggest_group: Vec<Point<f64>> = classifications
        .iter()
        .enumerate()
        .filter_map(|(i, value)| match value {
            dbscan::Classification::Core(num) if *num == group_num => {
                Some(Point::new(centroids[i][0], centroids[i][1]))
            }
            _ => None,
        })
        .collect();

    // And get the centroid of those points!
    let biggest_group = MultiPoint::new(biggest_group);
    biggest_group.centroid()
}
