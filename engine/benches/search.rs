use std::{
    io,
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use criterion::{criterion_group, criterion_main, Criterion};
use engine::{search, Position, SearchParams, HYPERBOLA_QUINTESSENCE_MOVE_GEN, POSITION_EVALUATOR};

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
                Arc::new(Mutex::new(io::sink())),
                Arc::new(AtomicBool::new(false)),
                None,
            )
        })
    });
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches);
