use backend::test_fixtures::NeighbourhoodFixture;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_build_map_model(c: &mut Criterion) {
    for neighbourhood in [
        NeighbourhoodFixture::BRISTOL_EAST,
        NeighbourhoodFixture::STRASBOURG,
    ] {
        // Do the file i/o (reading OSM.xml) outside of the bench loop
        let map_model_builder = neighbourhood.map_model_builder().unwrap();
        c.bench_function(
            &format!(
                "build map_model: {name}",
                name = neighbourhood.study_area_name
            ),
            |b| {
                b.iter(|| {
                    let map_model = map_model_builder().unwrap();
                    black_box(map_model);
                });
            },
        );
    }
}

criterion_group!(benches, benchmark_build_map_model);
criterion_main!(benches);
