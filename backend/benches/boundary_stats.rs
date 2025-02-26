use backend::boundary_stats::BoundaryStats;
use backend::test_fixtures::NeighbourhoodFixture;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_build_map_model(c: &mut Criterion) {
    for fixture in [NeighbourhoodFixture::DUNDEE] {
        // Do the file i/o (reading OSM.xml) outside of the bench loop
        let (neighbourhood, map) = fixture.neighbourhood_map().unwrap();
        assert_eq!(
            map.context_data.as_ref().unwrap().population_zones.len(),
            200
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
        c.bench_function(
            &format!(
                "generate auto boundaries (and stats) for all of {study_area}",
                study_area = fixture.study_area_name
            ),
            |b| {
                b.iter(|| {
                    let boundaries = map.render_auto_boundaries();
                    assert_eq!(boundaries.features.len(), 1102);
                    black_box(boundaries);
                });
            },
        );
    }
}

criterion_group!(benches, benchmark_build_map_model);
criterion_main!(benches);
