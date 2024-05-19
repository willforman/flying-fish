use std::{
    sync::{atomic::AtomicBool, mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

use engine::{search, Position, HYPERBOLA_QUINTESSENCE_MOVE_GEN, POSITION_EVALUATOR};

// Then, write test that engine stops searching soon after it's terminated

#[test]
fn test_search_terminates() {
    const DEPTH: u32 = 10;

    let terminate = Arc::new(AtomicBool::new(false));
    let (tx_best_move, rx_best_move) = mpsc::channel();

    let terminate_cloned = Arc::clone(&terminate);
    let handle = thread::spawn(move || {
        let best_move = search(
            &Position::start(),
            DEPTH,
            HYPERBOLA_QUINTESSENCE_MOVE_GEN,
            POSITION_EVALUATOR,
            Arc::clone(&terminate_cloned),
        );
        tx_best_move.send(best_move).unwrap();
    });

    thread::sleep(Duration::new(0, 100000));

    terminate.swap(true, std::sync::atomic::Ordering::Relaxed);

    let start_time = Instant::now();

    handle.join().unwrap();

    let duration = start_time.elapsed();
    assert!(duration < Duration::new(1, 0));

    let best_move = rx_best_move.recv().unwrap();
    assert_ne!(best_move, None);
}
