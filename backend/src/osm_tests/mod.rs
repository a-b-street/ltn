use crate::map_model::ProjectDetails;
use crate::test_fixtures::TEST_DB_SCHEMA_VERSION;
use crate::{FilterKind, Intersection, MapModel, RoadID};
use geo::MultiPolygon;

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
        .unwrap()
        .id;
    // It should have one turn restriction from south to west
    let restrictions = &map.turn_restrictions[i.0];
    assert_eq!(restrictions.len(), 1);
    assert!(map.get_r(restrictions[0].0).tags.is("name", "south"));
    assert!(map.get_r(restrictions[0].1).tags.is("name", "west"));
}

#[test]
fn test_dog_legs() {
    let map = load_osm_xml("dog_legs");

    // There should be one 4-way intersection
    let four_ways = map
        .intersections
        .iter()
        .filter(|i| i.roads.len() == 4)
        .collect::<Vec<_>>();
    assert_eq!(four_ways.len(), 1);
    assert_eq!(
        get_connected_roads(four_ways[0], &map),
        vec!["dog-leg 1", "dog-leg 2", "main", "main"]
    );

    // There should be two 3-way intersections
    let three_ways = map
        .intersections
        .iter()
        .filter(|i| i.roads.len() == 3)
        .collect::<Vec<_>>();
    assert_eq!(three_ways.len(), 2);
    for three_way in three_ways {
        let names = get_connected_roads(three_way, &map);
        assert!(
            names == vec!["dont transform 1", "main", "main"]
                || names == vec!["dont transform 2", "main", "main"]
        );
    }
}

#[test]
fn test_modal_filters() {
    let map = load_osm_xml("modal_filters");

    assert!(map
        .modal_filters
        .contains_key(&get_road_by_name(&map, "has barrier")));
    assert!(map
        .modal_filters
        .contains_key(&get_road_by_name(&map, "pedestrianized")));
    assert_eq!(
        FilterKind::BusGate,
        map.modal_filters[&get_road_by_name(&map, "bus only")].kind
    );

    assert!(!map
        .modal_filters
        .contains_key(&get_road_by_name(&map, "has kerb")));
    assert!(!map
        .modal_filters
        .contains_key(&get_road_by_name(&map, "not filtered")));

    assert_eq!(3, map.modal_filters.len());
}

pub fn load_osm_xml(filename: &str) -> MapModel {
    let path = format!(
        "{}/src/osm_tests/{filename}.osm.xml",
        env!("CARGO_MANIFEST_DIR")
    );
    let demand = None;
    let context_data = None;
    // No test cases need this
    let boundary_wgs84 = MultiPolygon::new(Vec::new());
    let mut map = MapModel::create_serialized(
        &std::fs::read(path).unwrap(),
        boundary_wgs84,
        demand,
        context_data,
    )
    .unwrap();
    map.finish_loading(ProjectDetails {
        project_name: "test-project".to_string(),
        study_area_name: None,
        app_focus: "global".to_string(),
        db_schema_version: TEST_DB_SCHEMA_VERSION,
    });
    map
}

fn get_road_by_name(map: &MapModel, name: &str) -> RoadID {
    map.roads
        .iter()
        .find(|r| r.tags.is("name", name))
        .expect(&format!("no road named {name}"))
        .id
}

fn get_connected_roads<'a>(intersection: &'a Intersection, map: &'a MapModel) -> Vec<&'a String> {
    let mut names: Vec<&String> = intersection
        .roads
        .iter()
        .map(|r| map.get_r(*r).tags.get("name").unwrap())
        .collect();
    names.sort();
    names
}
