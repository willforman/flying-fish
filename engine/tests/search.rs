use std::{
    io::Write,
    sync::{atomic::AtomicBool, mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use test_case::test_case;

use engine::Square::*;
use engine::{
    search, Move, Position, SearchParams, HYPERBOLA_QUINTESSENCE_MOVE_GEN, POSITION_EVALUATOR,
};
use testresult::TestResult;

#[derive(Clone, Debug)]
struct UCIResponseSaver {
    responses: Arc<Mutex<Vec<String>>>,
}

impl UCIResponseSaver {
    fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_new_responses(&self) -> Vec<String> {
        let mut responses = self.responses.lock().unwrap();
        let result = responses.clone();
        responses.clear();
        result
    }
}

impl Write for UCIResponseSaver {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let uci_res = String::from_utf8(buf.to_vec()).unwrap();
        self.responses.lock().unwrap().push(uci_res);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

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
            HYPERBOLA_QUINTESSENCE_MOVE_GEN,
            POSITION_EVALUATOR,
            Arc::new(Mutex::new(&mut UCIResponseSaver::new())),
            Arc::clone(&terminate_cloned),
            None,
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
#[test_case(Position::from_fen("rnbqkbnr/ppp2ppp/8/3pp3/4P1Q1/2N5/PPPP1PPP/R1B1KBNR b KQkq - 0 1").unwrap(), 3, Move::new(C8, G4) ; "obvious queen capture full board")]
fn test_finds_best_move(position: Position, max_depth: u64, best_move_want: Move) -> TestResult {
    let search_params = SearchParams {
        max_depth: Some(max_depth),
        ..SearchParams::default()
    };
    let response_saver = Arc::new(Mutex::new(UCIResponseSaver::new()));
    let (best_move_got, _) = search(
        &position,
        &search_params,
        HYPERBOLA_QUINTESSENCE_MOVE_GEN,
        POSITION_EVALUATOR,
        Arc::clone(&response_saver),
        Arc::new(AtomicBool::new(false)),
        None,
    )?;
    assert_eq!(best_move_got, Some(best_move_want));
    Ok(())
}

#[test_case(Position::from_fen("k7/8/1R6/8/8/8/8/1R1K4 w - - 0 1").unwrap(), Move::new(B6, B7) ; "rook ladder stalemate")]
fn test_doesnt_find_stalemate(position: Position, stalemate_move_dont_want: Move) -> TestResult {
    let search_params = SearchParams {
        max_depth: Some(1),
        ..SearchParams::default()
    };
    let response_saver = Arc::new(Mutex::new(UCIResponseSaver::new()));
    let (best_move_got, _) = search(
        &position,
        &search_params,
        HYPERBOLA_QUINTESSENCE_MOVE_GEN,
        POSITION_EVALUATOR,
        Arc::clone(&response_saver),
        Arc::new(AtomicBool::new(false)),
        None,
    )?;
    assert_ne!(best_move_got, Some(stalemate_move_dont_want));
    Ok(())
}
