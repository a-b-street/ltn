use geo::MultiPolygon;

use crate::MapModel;

#[test]
fn test_deadend_with_barrier() {
    // There should be one modal filter on "deadend"
    let map = load("deadend_with_barrier.osm.xml");
    assert_eq!(map.modal_filters.len(), 1);
    let road = map.get_r(*map.modal_filters.keys().next().unwrap());
    assert!(road.tags.is("name", "deadend"));
}

fn load(filename: &str) -> MapModel {
    let path = format!("{}/src/osm_tests/{filename}", env!("CARGO_MANIFEST_DIR"));
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
