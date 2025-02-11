use backend::test_fixtures::NeighbourhoodFixture;
use backend::Shortcuts;
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_shortcuts(c: &mut Criterion) {
    for neighbourhood_fixture in [
        NeighbourhoodFixture::BRISTOL_EAST,
        NeighbourhoodFixture::BRISTOL_WEST,
        NeighbourhoodFixture::STRASBOURG,
    ] {
        let (neighbourhood, map) = neighbourhood_fixture.neighbourhood_map().unwrap();
        c.bench_function(
            &format!(
                "shortcuts in {name}",
                name = neighbourhood_fixture.savefile_name
            ),
            |b| b.iter(|| Shortcuts::new(&map, &neighbourhood)),
        );
    }
}

criterion_group!(benches, benchmark_shortcuts);
criterion_main!(benches);
