use backend::test_fixtures::NeighbourhoodFixture;
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_impact_initial_build(c: &mut Criterion) {
    for fixture in [NeighbourhoodFixture::DUNDEE] {
        let mut map = fixture.map_model().unwrap();
        let fast_sample = false;

        c.benchmark_group(fixture.savefile_name)
            .sample_size(10)
            .bench_function("predict impact - initial build", |b| {
                b.iter(|| {
                    map.rebuild_router(1.0);
                    let mut impact = map.impact.take().unwrap();
                    impact.recalculate(&map, fast_sample);
                    map.impact = Some(impact);
                })
            });
    }
}

fn benchmark_impact_rebuild(c: &mut Criterion) {
    for fixture in [NeighbourhoodFixture::DUNDEE] {
        let mut map = fixture.map_model().unwrap();
        let fast_sample = false;

        // Do the first calculation (making the requests and counts_before) outside of the benchmark
        map.rebuild_router(1.0);
        {
            let mut impact = map.impact.take().unwrap();
            impact.recalculate(&map, fast_sample);
            map.impact = Some(impact);
        }

        c.benchmark_group(fixture.savefile_name)
            .sample_size(10)
            .bench_function("predict impact - rebuild", |b| {
                b.iter(|| {
                    let mut impact = map.impact.take().unwrap();
                    impact.invalidate_after_edits();
                    impact.recalculate(&map, fast_sample);
                    map.impact = Some(impact);
                })
            });
    }
}

criterion_group!(
    benches,
    benchmark_impact_rebuild,
    benchmark_impact_initial_build
);
criterion_main!(benches);
