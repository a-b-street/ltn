use std::collections::HashMap;

use anyhow::Result;
use geojson::GeoJson;

use crate::{
    Cell, MapModel, Neighbourhood, NeighbourhoodBoundary, NeighbourhoodDefinition, Shortcuts,
};

impl MapModel {
    pub fn get_all_traffic_predictions(&self) -> Result<String> {
        let mut traffic = HashMap::new();

        for boundary in self.generated_boundaries() {
            if let Ok(neighbourhood) = Neighbourhood::new(
                self,
                NeighbourhoodBoundary::new(
                    NeighbourhoodDefinition {
                        geometry: boundary.geometry,
                        name: String::new(),
                        waypoints: None,
                    },
                    None,
                ),
            ) {
                let cells = Cell::find_all(self, &neighbourhood);
                let shortcuts = Shortcuts::new(self, &neighbourhood, &cells);

                for r in &neighbourhood.interior_roads {
                    if shortcuts.count_per_road.contains_key(r) {
                        traffic.insert(*r, "medium");
                    } else {
                        traffic.insert(*r, "low");
                    }
                }
            }
        }

        let mut features = Vec::new();
        for road in &self.roads {
            let mut f = self.mercator.to_wgs84_gj(&road.linestring);
            // TODO Let the user edit this definition?
            if self.is_main_road[&road.id] {
                f.set_property("traffic", "high");
            } else if let Some(x) = traffic.remove(&road.id) {
                f.set_property("traffic", x);
            } else {
                warn!("{:?} has no traffic prediction", road.way);
            }
            features.push(f);
        }

        Ok(serde_json::to_string(&GeoJson::from(features))?)
    }
}
