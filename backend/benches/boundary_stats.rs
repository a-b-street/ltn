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
            &format!("build stats: {name}", name = fixture.study_area_name),
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
    }
}

criterion_group!(benches, benchmark_build_map_model);
criterion_main!(benches);
