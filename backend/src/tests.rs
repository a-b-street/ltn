use anyhow::Result;
use geojson::FeatureCollection;

use crate::test_fixtures::NeighbourhoodFixture;

#[test]
fn test_bristol_west() {
    test_example(&NeighbourhoodFixture::BRISTOL_WEST);
}

#[test]
fn test_bristol_east() {
    test_example(&NeighbourhoodFixture::BRISTOL_EAST);
}

#[test]
fn test_strasbourg() {
    test_example(&NeighbourhoodFixture::STRASBOURG);
}

fn test_example(neighbourhood_fixture: &NeighbourhoodFixture) {
    let output_path = format!(
        "../tests/output/{savefile_name}.geojson",
        savefile_name = neighbourhood_fixture.savefile_name
    );
    let expected = std::fs::read_to_string(&output_path).unwrap_or_else(|_| String::new());
    let actual = get_gj(&neighbourhood_fixture).unwrap();

    if expected != actual {
        std::fs::write(output_path, actual).unwrap();
        panic!("{savefile_name} has changed. Manually compare before/after changes with the web UI, then commit the output file to git.", savefile_name=neighbourhood_fixture.savefile_name);
    }
}

// TODO Must run from 'backend'
// TODO web/public/osm must be symlinked to local PBF copies
fn get_gj(neighbourhood_fixture: &NeighbourhoodFixture) -> Result<String> {
    let (neighbourhood, map) = neighbourhood_fixture.neighbourhood_map()?;
    let mut gj = prune_features(neighbourhood.to_gj(&map));
    // Include all existing modal filters anywhere in the map
    for mut f in map.filters_to_gj().features {
        f.set_property("kind", "existing_modal_filter");
        f.remove_property("road");
        f.remove_property("edited");
        f.remove_property("angle");
        gj.features.push(f);
    }

    Ok(serde_json::to_string(&gj)?)
}

// Remove OSM tags, for smaller files
fn prune_features(mut gj: FeatureCollection) -> FeatureCollection {
    // Remove border_arrows
    gj.features
        .retain(|f| f.property("kind").unwrap().as_str().unwrap() != "border_entries");

    for f in &mut gj.features {
        if matches!(
            f.geometry.as_ref().unwrap().value,
            geojson::Value::LineString(_)
        ) {
            let props = f.properties.as_mut().unwrap();
            props.retain(|k, _| {
                ["travel_flow", "kind", "pct", "shortcuts", "way"].contains(&k.as_str())
            });
        }
    }
    gj
}
