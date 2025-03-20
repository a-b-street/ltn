use backend::boundary_stats::BoundaryStats;
use backend::test_fixtures::NeighbourhoodFixture;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_build_map_model(c: &mut Criterion) {
    for (fixture, expected_population_zones, expected_generated_boundaries) in [
        (NeighbourhoodFixture::INVERNESS, 325, 9513),
        (NeighbourhoodFixture::DUNDEE, 200, 240),
    ] {
        // Do the file i/o (reading OSM.xml) outside of the bench loop
        let (neighbourhood, map) = fixture.neighbourhood_map().unwrap();
        assert_eq!(
            map.context_data.as_ref().unwrap().population_zones.len(),
            expected_population_zones
        );
        c.bench_function(
            &format!(
                "build stats: {neighbourhood} in {study_area}",
                neighbourhood = fixture.neighbourhood_name,
                study_area = fixture.study_area_name
            ),
            |b| {
                b.iter(|| {
                    let stats = BoundaryStats::new(
                        neighbourhood.boundary.geometry(),
                        map.context_data.as_ref(),
                    );
                    black_box(stats);
                });
            },
        );
        c.benchmark_group(fixture.savefile_name)
            .sample_size(fixture.bench_sample_size())
            .bench_function("generate auto boundaries (and stats)", |b| {
                b.iter(|| {
                    let boundaries = map.generated_boundaries();
                    assert_eq!(boundaries.features.len(), expected_generated_boundaries);
                    black_box(boundaries);
                });
            });
    }
}

criterion_group!(benches, benchmark_build_map_model);
criterion_main!(benches);
