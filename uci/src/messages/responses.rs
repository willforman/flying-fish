use std::io::{self, Write};

use engine::position::Move;

pub trait WriteUCIResponse {
    fn write_uci_response(&self, uci_response: String);
}

pub struct UCIResponseStdoutWriter;

impl WriteUCIResponse for UCIResponseStdoutWriter {
    fn write_uci_response(&self, uci_response: String) {
        io::stdout().write(uci_response.as_bytes()).unwrap();
    }
}

#[derive(Debug)]
pub(crate) enum UCIResponse {
    ID {
        name: Option<String>,
        author: Option<String>,
    },
    UCIOk,
    ReadyOk,
    BestMove {
        mve: Move,
        ponder: Option<Move>,
    },
    Info {
        info: Info,
    },
    Option {
        option: UCIOption,
    },
}

#[derive(Debug)]
pub enum Info {
    Depth {
        str: String,
    },
    Seldepth {
        str: String,
    },
    Time {
        str: String,
    },
    Nodes {
        str: String,
    },
    PV {
        moves: Vec<Move>,
    },
    MultiPV {
        num: i32,
    },
    Score {
        str: String,
    },
    CurrMove {
        mve: Move,
    },
    CurrMoveNumber {
        move_num: u32,
    },
    HashFull {
        num_per_mill: u32,
    },
    NPS {
        nodes_per_second: f32,
    },
    TBHits {
        positions_found: u32,
    },
    SBHits {
        positions_found: u32,
    },
    CPULoad {
        cpu_usage: f32,
    },
    String {
        str: String,
    },
    Refutation {
        start_move: Move,
        line: Vec<Move>,
    },
    CurrLine {
        cpu_num: Option<u8>,
        line: Vec<Move>,
    },
}

enum Score {
    Cp { score: f32 },
    Mate { num_moves: u8 },
    LowerBound,
    UpperBound,
}

#[derive(Debug)]
pub struct UCIOption {
    name: String,
    type_: UCIOptionType,
    default: Option<String>,
}

#[derive(Debug)]
pub enum UCIOptionType {
    Check,
    Spin { range_start: i32, range_end: i32 },
    Combo { options: Vec<String> },
    Button,
    String { str: String },
}

impl Into<String> for UCIResponse {
    fn into(self) -> String {
        "test".to_string()
    }
}
