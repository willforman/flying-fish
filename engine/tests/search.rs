use std::{
    io::Write,
    sync::{atomic::AtomicBool, mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use engine::{search, Position, SearchParams, HYPERBOLA_QUINTESSENCE_MOVE_GEN, POSITION_EVALUATOR};

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
        )
        .unwrap();
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
