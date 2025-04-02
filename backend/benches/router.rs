use backend::od::synthetic_od_requests;
use backend::test_fixtures::NeighbourhoodFixture;
use backend::Router;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_build_router(c: &mut Criterion) {
    for neighbourhood in [
        NeighbourhoodFixture::BRISTOL_EAST,
        NeighbourhoodFixture::STRASBOURG,
    ] {
        let map = neighbourhood.map_model().unwrap();
        let routing_input_before = map.router_input_before();
        let main_road_penalty = 1.0;

        c.benchmark_group(neighbourhood.savefile_name)
            .sample_size(neighbourhood.bench_sample_size())
            .bench_function("build router", |b| {
                b.iter(|| {
                    let router = Router::new(&routing_input_before, main_road_penalty);
                    black_box(router);
                });
            });
    }
}

fn benchmark_route(c: &mut Criterion) {
    for neighbourhood in [
        NeighbourhoodFixture::BRISTOL_EAST,
        NeighbourhoodFixture::STRASBOURG,
    ] {
        let map = neighbourhood.map_model().unwrap();
        let main_road_penalty = 1.0;
        let router_input = map.router_input_before();
        let router = Router::new(&router_input, main_road_penalty);

        let route_requests = synthetic_od_requests(&map);
        c.benchmark_group(neighbourhood.savefile_name)
            .sample_size(neighbourhood.bench_sample_size())
            .bench_function("routing", |b| {
                b.iter(|| {
                    let mut num_found = 0;
                    for (start, end, _) in &route_requests {
                        if let Some(_found) = router.route_from_roads(&router_input, *start, *end) {
                            num_found += 1;
                        }
                    }
                    match neighbourhood {
                        // These exact numbers are brittle - but they should only change if the
                        // routing logic or the input data are updated, and even then they shouldn't
                        // change by much.
                        NeighbourhoodFixture::BRISTOL_EAST => assert_eq!(num_found, 830),
                        NeighbourhoodFixture::STRASBOURG => assert_eq!(num_found, 885),
                        _ => todo!(
                            "unknown neighbourhood: {neighbourhood:?}, (num_found: {num_found})"
                        ),
                    }
                });
            });
    }
}

criterion_group!(benches, benchmark_build_router, benchmark_route);
criterion_main!(benches);
