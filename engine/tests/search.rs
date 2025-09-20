use std::{
    io::Write,
    sync::{Arc, Mutex, atomic::AtomicBool, mpsc},
    thread,
    time::{Duration, Instant},
};
use test_case::test_case;

use engine::Square::*;
use engine::{
    MOVE_GEN, Move, POSITION_EVALUATOR, Position, SearchParams, TranspositionTable, search,
};
use testresult::TestResult;

#[test]
fn test_search_terminates() {
    let terminate = Arc::new(AtomicBool::new(false));
    let (tx_best_move, rx_best_move) = mpsc::channel();

    let terminate_cloned = Arc::clone(&terminate);
    let handle = thread::spawn(move || {
        let (best_move, _) = search(
            &Position::start(),
            &SearchParams {
                move_time: Some(Duration::from_secs(2)),
                ..SearchParams::default()
            },
            MOVE_GEN,
            POSITION_EVALUATOR,
            &mut TranspositionTable::new(),
            Arc::clone(&terminate_cloned),
        )
        .unwrap();
        tx_best_move.send(best_move).unwrap();
    });

    thread::sleep(Duration::from_millis(25));

    terminate.swap(true, std::sync::atomic::Ordering::Relaxed);

    let start_time = Instant::now();

    handle.join().unwrap();

    let duration = start_time.elapsed();
    assert!(duration < Duration::new(1, 0));

    let best_move = rx_best_move.recv().unwrap();
    assert_ne!(best_move, None);
}

#[test_case(Position::from_fen("k7/6R1/7R/8/8/8/8/3K4 w - - 0 1").unwrap(), 1, Move::new(H6, H8) ; "rook ladder in 1 white")]
#[test_case(Position::from_fen("8/k7/8/8/8/1r6/r7/7K b - - 0 1").unwrap(), 1, Move::new(B3, B1) ; "rook ladder in 1 black")]
#[test_case(Position::from_fen("1k6/8/2R5/7R/8/8/8/6K1 w - - 0 1").unwrap(), 3, Move::new(H5, H7) ; "rook ladder in 3 white")]
#[test_case(Position::from_fen("8/k7/8/8/r7/5r2/8/6K1 b - - 0 1").unwrap(), 3, Move::new(A4, A2) ; "rook ladder in 3 black")]
#[test_case(Position::from_fen("2k5/q7/8/8/8/8/8/6QK w - - 0 1").unwrap(), 3, Move::new(G1, A7) ; "obvious queen capture empty board")]
#[test_case(Position::from_fen("rnbqkbnr/ppp2ppp/8/3pp3/4P1Q1/2N5/PPPP1PPP/R1B1KBNR b KQkq - 0 1").unwrap(), 1, Move::new(C8, G4) ; "obvious queen capture full board depth 1")]
#[test_case(Position::from_fen("rnbqkbnr/ppp2ppp/8/3pp3/4P1Q1/2N5/PPPP1PPP/R1B1KBNR b KQkq - 0 1").unwrap(), 3, Move::new(C8, G4) ; "obvious queen capture full board depth 3")]
#[test_case(Position::from_fen("7k/8/8/8/8/3r4/4r3/1K6 w - - 0 1").unwrap(), 3, Move::new(B1, C1) ; "obvious move to avoid mate")]
fn test_finds_best_move(position: Position, max_depth: u8, best_move_want: Move) -> TestResult {
    let search_params = SearchParams {
        max_depth: Some(max_depth),
        move_time: Some(Duration::from_secs(10)),
        ..SearchParams::default()
    };
    let (best_move_got, _) = search(
        &position,
        &search_params,
        MOVE_GEN,
        POSITION_EVALUATOR,
        &mut TranspositionTable::new(),
        Arc::new(AtomicBool::new(false)),
    )?;
    assert_eq!(best_move_got, Some(best_move_want));
    Ok(())
}

#[test_case(Position::from_fen("k7/8/1R6/8/8/8/8/1R1K4 w - - 0 1").unwrap(), Move::new(B6, B7) ; "rook ladder stalemate")]
fn test_doesnt_find_stalemate(position: Position, stalemate_move_dont_want: Move) -> TestResult {
    let tt = TranspositionTable::new();
    let search_params = SearchParams {
        max_depth: Some(1),
        ..SearchParams::default()
    };
    let (best_move_got, _) = search(
        &position,
        &search_params,
        MOVE_GEN,
        POSITION_EVALUATOR,
        &mut TranspositionTable::new(),
        Arc::new(AtomicBool::new(false)),
    )?;
    assert_ne!(best_move_got, Some(stalemate_move_dont_want));
    assert_ne!(best_move_got, None);
    Ok(())
}

#[test_case(Position::from_fen("7k/8/4q3/8/8/4R3/5P2/K7 b - - 0 1").unwrap(), Move::new(E6, E3))]
fn test_avoids_horizon_effect(position: Position, horizon_effect_move: Move) -> TestResult {
    let search_params = SearchParams {
        max_depth: Some(1),
        ..SearchParams::default()
    };
    let (best_move_got, _) = search(
        &position,
        &search_params,
        MOVE_GEN,
        POSITION_EVALUATOR,
        &mut TranspositionTable::new(),
        Arc::new(AtomicBool::new(false)),
    )?;

    assert_ne!(best_move_got, Some(horizon_effect_move));
    assert_ne!(best_move_got, None);
    Ok(())
}
