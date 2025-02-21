use backend::test_fixtures::NeighbourhoodFixture;
use backend::Neighbourhood;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_build_neighbourhood(c: &mut Criterion) {
    for neighbourhood in [
        NeighbourhoodFixture::BRISTOL_EAST,
        NeighbourhoodFixture::BRISTOL_WEST,
        NeighbourhoodFixture::STRASBOURG,
    ] {
        let (neighbourhood_stats, map) = neighbourhood.neighbourhood_params().unwrap();
        let edit_perimeter_roads = false;
        c.bench_function(
            &format!(
                "build neighbourhood: {name}",
                name = neighbourhood.savefile_name
            ),
            |b| {
                b.iter(|| {
                    let neighbourhood =
                        Neighbourhood::new(&map, neighbourhood_stats.clone(), edit_perimeter_roads)
                            .unwrap();
                    black_box(neighbourhood);
                });
            },
        );
    }
}

criterion_group!(benches, benchmark_build_neighbourhood);
criterion_main!(benches);
