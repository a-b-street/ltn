use geo::MultiPolygon;

use crate::MapModel;

#[test]
fn test_deadend_with_barrier() {
    // There should be one modal filter on "deadend"
    let map = load_osm_xml("deadend_with_barrier");
    assert_eq!(map.modal_filters.len(), 1);
    let road = map.get_r(*map.modal_filters.keys().next().unwrap());
    assert!(road.tags.is("name", "deadend"));
}

#[test]
fn test_no_left_turn() {
    let map = load_osm_xml("no_left_turn");
    // Find the main intersection
    let i = map
        .intersections
        .iter()
        .find(|i| i.roads.len() > 1)
        .unwrap();
    // It should have one turn restriction from south to west
    assert_eq!(i.turn_restrictions.len(), 1);
    assert!(map.get_r(i.turn_restrictions[0].0).tags.is("name", "south"));
    assert!(map.get_r(i.turn_restrictions[0].1).tags.is("name", "west"));
}

pub fn load_osm_xml(filename: &str) -> MapModel {
    let path = format!(
        "{}/src/osm_tests/{filename}.osm.xml",
        env!("CARGO_MANIFEST_DIR")
    );
    let demand = None;
    // No test cases need this
    let boundary_wgs84 = MultiPolygon::new(Vec::new());
    let study_area_name = None;
    MapModel::new(
        &std::fs::read(path).unwrap(),
        boundary_wgs84,
        study_area_name,
        demand,
    )
    .unwrap()
}
