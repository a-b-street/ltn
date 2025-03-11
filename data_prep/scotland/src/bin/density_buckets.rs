use data_prep::PopulationZoneInput;

/// Print the Quintile boundaries for all of Scotland's population zones
fn main() {
    let mut population_zones = PopulationZoneInput::read_all_from_file().unwrap();
    population_zones.sort_by(|zone_a, zone_b| {
        zone_a
            .density_per_km2()
            .partial_cmp(&zone_b.density_per_km2())
            .unwrap()
    });

    let buckets = 5;
    let bucket_width = population_zones.len() / buckets;

    let limits: Vec<_> = (0..=buckets)
        .map(|bucket| {
            let limit_idx = bucket * bucket_width;
            population_zones[limit_idx]
                .density_per_km2()
                .round() as u64
        })
        .collect();

    println!("most_dense: {}", population_zones.iter().rev().next().unwrap().density_per_km2());
    // > Raw limits for Scotland density (/ km²): [0, 1324, 2940, 4247, 5858, 52389]
    println!("Raw limits for Scotland density (/ km²): {limits:?}");
}
