use std::sync::{Arc, atomic::AtomicBool};
use std::time::Instant;

use criterion::{Criterion, criterion_group, criterion_main};
use engine::{MOVE_GEN, POSITION_EVALUATOR, Position, SearchParams, perft, search};

const PERFT_BENCHMARK_FENS_AND_DEPTHS: &[(&str, usize, &str)] = &[
    (
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        5,
        "starting position",
    ),
    (
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        4,
        "middlegame",
    ),
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 5, "endgame"),
];

pub fn benchmark_perft(c: &mut Criterion) {
    let mut group = c.benchmark_group("perft");
    group.sample_size(15);

    for (fen, depth, position_name) in PERFT_BENCHMARK_FENS_AND_DEPTHS.iter() {
        let pos = Position::from_fen(fen).unwrap();
        let bench_name = format!("perft {}", position_name);

        group.bench_function(&bench_name, |b| {
            b.iter_custom(|iters| {
                let start = Instant::now();
                let mut total_nodes = 0;

                for _ in 0..iters {
                    let (_, nodes) = perft(&pos, *depth, MOVE_GEN);
                    total_nodes += nodes;
                }

                let elapsed = start.elapsed();
                let nps = total_nodes as f64 / elapsed.as_secs_f64();
                println!("{} (depth {}): {:.0} nodes/second", &bench_name, depth, nps);

                elapsed
            })
        });
    }
}

const SEARCH_BENCHMARK_FENS_AND_DEPTHS: &[(&str, usize, &str)] = &[
    (
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        4,
        "starting position",
    ),
    (
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        2,
        "middlegame",
    ),
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 5, "endgame"),
];

pub fn benchmark_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("search");
    group.sample_size(15);

    for (fen, depth, position_name) in SEARCH_BENCHMARK_FENS_AND_DEPTHS.iter() {
        let pos = Position::from_fen(fen).unwrap();
        let bench_name = format!("search {}", position_name);
        let search_params = SearchParams {
            max_depth: Some(*depth as u64),
            ..SearchParams::default()
        };

        group.bench_function(&bench_name, |b| {
            b.iter_custom(|iters| {
                let start = Instant::now();
                let mut total_nodes = 0;

                for _ in 0..iters {
                    let (_, search_result) = search(
                        &pos,
                        &search_params,
                        MOVE_GEN,
                        POSITION_EVALUATOR,
                        Arc::new(AtomicBool::new(false)),
                    )
                    .unwrap();
                    total_nodes += search_result.positions_processed;
                }

                let elapsed = start.elapsed();
                let nps = total_nodes as f64 / elapsed.as_secs_f64();
                println!("{} (depth {}): {:.0} nodes/second", &bench_name, depth, nps);

                elapsed
            })
        });
    }
}

criterion_group!(benches, benchmark_perft, benchmark_search);
criterion_main!(benches);
