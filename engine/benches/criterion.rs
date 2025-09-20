use std::sync::{Arc, atomic::AtomicBool};
use std::time::{Duration, Instant};

use criterion::{Criterion, criterion_group, criterion_main};
use engine::{
    MOVE_GEN, POSITION_EVALUATOR, Position, SearchParams, TranspositionTable, perft, search,
};

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
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 6, "endgame"),
];

pub fn benchmark_perft(c: &mut Criterion) {
    const SAMPLE_SIZE: usize = 30;
    let mut group = c.benchmark_group("perft");
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(Duration::from_secs(10));

    let mut nps_vals = Vec::with_capacity(PERFT_BENCHMARK_FENS_AND_DEPTHS.len());

    for (fen, depth, position_name) in PERFT_BENCHMARK_FENS_AND_DEPTHS.iter() {
        let pos = Position::from_fen(fen).unwrap();
        let bench_name = format!("perft {}", position_name);

        let mut pos_nps_vals = Vec::with_capacity(SAMPLE_SIZE);

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
                println!("{:.0} nodes/second", nps);
                pos_nps_vals.push(nps);

                elapsed
            })
        });

        let pos_nps_sum: f64 = pos_nps_vals.iter().copied().sum();
        let pos_nps_avg = pos_nps_sum / pos_nps_vals.len() as f64;
        println!(
            "{} (depth {}) overall nps: {:.0} nodes/second",
            position_name, depth, pos_nps_avg
        );
        nps_vals.push(pos_nps_avg);
    }

    let nps_sum: f64 = nps_vals.iter().copied().sum();
    let nps_avg = nps_sum / nps_vals.len() as f64;
    println!("average nps: {:.0} nodes/second", nps_avg);
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
    group.sample_size(30);
    group.warm_up_time(Duration::from_secs(10));

    for (fen, depth, position_name) in SEARCH_BENCHMARK_FENS_AND_DEPTHS.iter() {
        let pos = Position::from_fen(fen).unwrap();
        let bench_name = format!("search {}", position_name);
        let search_params = SearchParams {
            max_depth: Some(*depth as u8),
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
                        &mut TranspositionTable::new(),
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
