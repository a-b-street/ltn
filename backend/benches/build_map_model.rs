use backend::test_fixtures::NeighbourhoodFixture;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_build_map_model(c: &mut Criterion) {
    for neighbourhood in [
        NeighbourhoodFixture::BRISTOL_EAST,
        NeighbourhoodFixture::STRASBOURG,
    ] {
        // Do the file i/o (reading OSM.xml) outside of the bench loop
        let map_model_builder = neighbourhood.map_model_builder().unwrap();
        c.benchmark_group(neighbourhood.savefile_name)
            .sample_size(neighbourhood.bench_sample_size())
            .bench_function("build map_model", |b| {
                b.iter(|| {
                    let map_model = map_model_builder().unwrap();
                    black_box(map_model);
                });
            });
    }
}

criterion_group!(benches, benchmark_build_map_model);
criterion_main!(benches);
