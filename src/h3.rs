use geo;
use h3o::{
    geom::{PolyfillConfig, Polygon, ToCells},
    CellIndex, Resolution,
};

use crate::data::Activity;

pub fn polyfill(linestring: &geo::LineString) -> Vec<CellIndex> {
    let polygon = geo::Polygon::new(linestring.to_owned(), vec![]);
    let polygon = Polygon::from_degrees(polygon).unwrap();
    let cells = polygon
        .to_cells(PolyfillConfig::new(Resolution::Seven))
        .collect::<Vec<_>>();
    cells
}

pub fn polyfill_all(activities: &Vec<Activity>) -> Vec<CellIndex> {
    let mut all_cells: Vec<CellIndex> = Vec::new();
    for activity in activities {
        let cells = match &activity.linestring {
            Some(ls) => polyfill(ls),
            None => continue,
        };
        all_cells.extend(cells);
    }
    println!("{:?}", all_cells);
    all_cells
}
