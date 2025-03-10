use backend::test_fixtures::NeighbourhoodFixture;
use backend::Neighbourhood;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_build_neighbourhood(c: &mut Criterion) {
    for neighbourhood in [
        NeighbourhoodFixture::BRISTOL_EAST,
        NeighbourhoodFixture::BRISTOL_WEST,
        NeighbourhoodFixture::STRASBOURG,
    ] {
        let (neighbourhood_boundary, map) = neighbourhood.neighbourhood_params().unwrap();
        let edit_perimeter_roads = false;
        c.benchmark_group(neighbourhood.savefile_name)
            .sample_size(neighbourhood.bench_sample_size())
            .bench_function("build neighbourhood", |b| {
                b.iter(|| {
                    let neighbourhood = Neighbourhood::new(
                        &map,
                        neighbourhood_boundary.clone(),
                        edit_perimeter_roads,
                    )
                    .unwrap();
                    black_box(neighbourhood);
                });
            });
    }
}

criterion_group!(benches, benchmark_build_neighbourhood);
criterion_main!(benches);
