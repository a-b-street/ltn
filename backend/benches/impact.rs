use backend::test_fixtures::NeighbourhoodFixture;
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_impact(c: &mut Criterion) {
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

        c.bench_function("predict impact", |b| {
            b.iter(|| {
                let mut impact = map.impact.take().unwrap();
                impact.invalidate_after_edits();
                impact.recalculate(&map, fast_sample);
                map.impact = Some(impact);
            })
        });
    }
}

criterion_group!(benches, benchmark_impact);
criterion_main!(benches);
