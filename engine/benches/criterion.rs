use std::sync::{atomic::AtomicBool, Arc};

use criterion::{criterion_group, criterion_main, Criterion};
use engine::{
    perft, search, Position, SearchParams, HYPERBOLA_QUINTESSENCE_MOVE_GEN, POSITION_EVALUATOR,
};

pub fn benchmark_perft(c: &mut Criterion) {
    let mut group = c.benchmark_group("perft");
    group.sample_size(400);

    let pos = Position::start();

    group.bench_function("perft early game", |b| {
        b.iter(|| {
            perft(&pos, 4, HYPERBOLA_QUINTESSENCE_MOVE_GEN);
        })
    });
}

pub fn benchmark_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("search");
    group.sample_size(400);

    let pos = Position::start();
    let search_params = SearchParams {
        max_depth: Some(4),
        ..SearchParams::default()
    };

    group.bench_function("search early game", |b| {
        b.iter(|| {
            search(
                &pos,
                &search_params,
                HYPERBOLA_QUINTESSENCE_MOVE_GEN,
                POSITION_EVALUATOR,
                Arc::new(AtomicBool::new(false)),
            )
        })
    });
}

criterion_group!(benches, benchmark_perft, benchmark_search);
criterion_main!(benches);
