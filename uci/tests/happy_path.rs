use std::{
    io::Write,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use engine::{AUTHOR, HYPERBOLA_QUINTESSENCE_MOVE_GEN, NAME};
use uci::UCI;

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
fn test_happy_path() {
    let move_gen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;
    let response_saver = Arc::new(Mutex::new(UCIResponseSaver::new()));
    let mut uci = UCI::new(move_gen, Arc::clone(&response_saver));

    uci.handle_command("uci").unwrap();

    let responses = response_saver.lock().unwrap().get_new_responses();

    let id_name = format!("id name {}", NAME);
    let id_author = format!("id author {}", AUTHOR);

    assert_eq!(&responses[0..2], &[id_name, id_author]);
    assert_eq!(responses.last().unwrap(), "uciok");

    uci.handle_command("debug on").unwrap();

    for resp in responses[2..responses.len() - 1].iter() {
        assert!(resp.starts_with("option"));
    }

    uci.handle_command("isready").unwrap();

    let responses = response_saver.lock().unwrap().get_new_responses();
    assert_eq!(responses, vec!["readyok"]);

    uci.handle_command("ucinewgame").unwrap();
    uci.handle_command("go infinite").unwrap();

    thread::sleep(Duration::new(0, 1_000_000)); // 1 ms
    uci.handle_command("stop").unwrap();
    thread::sleep(Duration::new(0, 1_000_000));

    let responses = response_saver.lock().unwrap().get_new_responses();
    assert_eq!(responses, vec!["bestmove b2b4"]);
}
