use geo;
use h3o::{
    geom::{LineString, PolyfillConfig, ToCells},
    CellIndex, Resolution,
};

use crate::data::Activity;

fn polyfill(linestring: &geo::LineString) -> Vec<CellIndex> {
    let coords: Vec<geo::Coord> = linestring.to_owned().into_inner();
    let linestring = geo::LineString::new(coords);
    let linestring = LineString::from_degrees(linestring).unwrap();
    let cells = linestring
        .to_cells(PolyfillConfig::new(Resolution::Nine))
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
    all_cells
}
