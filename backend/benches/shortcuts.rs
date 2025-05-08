use backend::test_fixtures::NeighbourhoodFixture;
use backend::{Cell, Shortcuts};
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_shortcuts(c: &mut Criterion) {
    for neighbourhood_fixture in [
        NeighbourhoodFixture::BRISTOL_EAST,
        NeighbourhoodFixture::BRISTOL_WEST,
        NeighbourhoodFixture::STRASBOURG,
    ] {
        let (neighbourhood, map) = neighbourhood_fixture.neighbourhood_map().unwrap();
        let cells = Cell::find_all(&map, &neighbourhood);

        c.bench_function(
            &format!(
                "shortcuts in {name}",
                name = neighbourhood_fixture.savefile_name
            ),
            |b| b.iter(|| Shortcuts::new(&map, &neighbourhood, &cells)),
        );
    }
}

criterion_group!(benches, benchmark_shortcuts);
criterion_main!(benches);
