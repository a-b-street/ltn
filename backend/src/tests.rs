use anyhow::Result;
use geo::Polygon;
use geojson::FeatureCollection;

use crate::{MapModel, Neighbourhood};

#[test]
fn test_bristol_west() {
    test_example("bristol", "bristol_west", "west");
}

fn test_example(study_area_name: &str, savefile_name: &str, neighbourhood_name: &str) {
    let output_path = format!("../tests/output/{savefile_name}.geojson");
    let expected = std::fs::read_to_string(&output_path).unwrap_or_else(|_| String::new());
    let actual = get_gj(study_area_name, savefile_name, neighbourhood_name).unwrap();

    if expected != actual {
        std::fs::write(output_path, actual).unwrap();
        panic!("{savefile_name} has changed. Manually compare before/after changes with the web UI, then commit the output file to git.");
    }
}

// TODO Must run from 'backend'
// TODO web/public/osm must be symlinked to local PBF copies
fn get_gj(study_area_name: &str, savefile_name: &str, neighbourhood_name: &str) -> Result<String> {
    let input_bytes = std::fs::read(format!("../web/public/osm/{study_area_name}.pbf"))?;
    let mut map = MapModel::new(&input_bytes, Some(study_area_name.to_string()))?;

    let savefile: FeatureCollection =
        std::fs::read_to_string(format!("../tests/{savefile_name}.geojson"))?.parse()?;
    map.load_savefile(savefile)?;

    // set_current_neighbourhood equivalent
    let boundary_gj = map.boundaries.get(neighbourhood_name).cloned().unwrap();
    let mut boundary_geo: Polygon = boundary_gj.try_into()?;
    map.mercator.to_mercator_in_place(&mut boundary_geo);
    let neighbourhood = Neighbourhood::new(&map, neighbourhood_name.to_string(), boundary_geo)?;

    // TODO Include modal filters, once we detect existing ones

    Ok(serde_json::to_string_pretty(&neighbourhood.to_gj(&map))?)
}
