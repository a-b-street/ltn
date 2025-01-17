use backend::test_fixtures::NeighbourhoodFixture;
use backend::Neighbourhood;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_build_neighbourhood(c: &mut Criterion) {
    for neighbourhood in [
        NeighbourhoodFixture::BRISTOL_EAST,
        NeighbourhoodFixture::BRISTOL_WEST,
        NeighbourhoodFixture::STRASBOURG,
    ] {
        c.bench_function(
            &format!(
                "build neighbourhood: {name}",
                name = neighbourhood.savefile_name
            ),
            |b| {
                let (map, boundary_geo) = neighbourhood.neighbourhood_params().unwrap();
                let edit_perimeter_roads = false;
                b.iter(|| {
                    let neighbourhood = Neighbourhood::new(
                        &map,
                        neighbourhood.neighbourhood_name.to_string(),
                        boundary_geo.clone(),
                        edit_perimeter_roads,
                    )
                    .unwrap();
                    black_box(neighbourhood);
                });
            },
        );
    }
}

criterion_group!(benches, benchmark_build_neighbourhood);
criterion_main!(benches);
