use std::{
    borrow::Borrow,
    cell::RefCell,
    sync::{Arc, Mutex, RwLock},
};

use engine::HYPERBOLA_QUINTESSENCE_MOVE_GEN;
use uci::{WriteUCIResponse, UCI};

struct UCIResponseSaver {
    responses: Arc<Mutex<Vec<String>>>,
}

impl UCIResponseSaver {
    fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_responses(&self) -> Vec<String> {
        self.responses.lock().unwrap().clone()
    }
}

impl WriteUCIResponse for UCIResponseSaver {
    fn write_uci_response(&self, uci_response: String) {
        self.responses.lock().unwrap().push(uci_response);
    }
}

#[test]
fn test_happy_path() {
    let move_gen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;

    let response_saver = Arc::new(UCIResponseSaver::new());

    let mut uci = UCI::new(move_gen, Arc::clone(&response_saver));
    uci.handle_command("uci".to_string()).unwrap();

    let responses = response_saver.get_responses();
    let a: Vec<String> = Vec::new();
    assert_eq!(responses, a);
}
