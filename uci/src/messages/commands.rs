use core::fmt;
use std::fmt::Display;
use std::io;
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use engine::{Move, SearchParams, Side, Square};
use winnow::ascii::{alphanumeric1, digit1};
use winnow::combinator::{alt, opt, preceded, rest, separated, terminated};
use winnow::token::{one_of, take_until, take_while};
use winnow::{PResult, Parser};

pub trait ReadUCICommand {
    fn read_uci_command(&self) -> Result<String>;
}

pub struct UCICommandStdinReader;

impl ReadUCICommand for UCICommandStdinReader {
    fn read_uci_command(&self) -> Result<String> {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        Ok(buffer)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum UCICommand {
    #[allow(clippy::upper_case_acronyms)]
    UCI,
    Debug {
        on: bool,
    },
    IsReady,
    SetOption {
        name: String,
        value: Option<String>,
    },
    Register {
        name: String,
        code: String,
    },
    RegisterLater,
    UCINewGame,
    Position {
        fen: Option<String>,
        moves: Option<Vec<Move>>,
    },
    Go {
        params: SearchParams,
    },
    Stop,
    PonderHit,
    Quit,

    // Non standard UCI commands
    Eval,
    Perft {
        depth: usize,
    },
    PerftFull {
        depth: usize,
    },
    PerftBenchmark,
}

impl fmt::Display for UCICommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!();
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum GoParameter {
    SearchMoves { moves: Vec<Move> },
    Time { time: Duration, side: Side },
    Inc { time: Duration, side: Side },
    MovesToGo { moves: u16 },
    Depth { moves: u64 },
    Nodes { nodes: u64 },
    Mate { moves: u64 },
    MoveTime { time: Duration },
    Infinite,
    Ponder,
}

impl fmt::Display for GoParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!();
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct UCICommandParseError(String);

impl Display for UCICommandParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for UCICommandParseError {}

impl FromStr for UCICommand {
    type Err = UCICommandParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        alt((
            // parse_ucinewgame must go before parse_uci because they share prefix
            parse_ucinewgame,
            parse_uci,
            parse_debug,
            parse_isready,
            parse_setoption,
            parse_register,
            parse_register_later,
            parse_position,
            parse_stop,
            parse_ponderhit,
            parse_quit,
            parse_eval,
            parse_perft,
            parse_perft_full,
            parse_perft_benchmark,
            parse_go,
        ))
        .parse(input)
        .map_err(|_| UCICommandParseError(format!("cannot parse: [{}]", input)))
    }
}

// ======================================================
// Winnow Parsing functions (non go commands)
// ======================================================

fn parse_uci(input: &mut &str) -> PResult<UCICommand> {
    "uci".parse_next(input).map(|_| UCICommand::UCI)
}

fn parse_debug(input: &mut &str) -> PResult<UCICommand> {
    preceded("debug ", alt(("on".value(true), "off".value(false))))
        .parse_next(input)
        .map(|on| UCICommand::Debug { on })
}

fn parse_isready(input: &mut &str) -> PResult<UCICommand> {
    "isready".parse_next(input).map(|_| UCICommand::IsReady)
}

fn parse_setoption(input: &mut &str) -> PResult<UCICommand> {
    preceded(
        "setoption name ",
        alt((
            (take_until(0.., " value"), preceded(" value ", rest)).map(
                |(name, value): (&str, &str)| UCICommand::SetOption {
                    name: name.to_string(),
                    value: Some(value.to_string()),
                },
            ),
            rest.map(|name: &str| UCICommand::SetOption {
                name: name.to_string(),
                value: None,
            }),
        )),
    )
    .parse_next(input)
}

fn parse_register(input: &mut &str) -> PResult<UCICommand> {
    preceded(
        "register name ",
        (take_until(0.., " code"), preceded(" code ", rest)).map(|(name, code): (&str, &str)| {
            UCICommand::Register {
                name: name.to_string(),
                code: code.to_string(),
            }
        }),
    )
    .parse_next(input)
}

fn parse_register_later(input: &mut &str) -> PResult<UCICommand> {
    "register later"
        .value(UCICommand::RegisterLater)
        .parse_next(input)
}

fn parse_ucinewgame(input: &mut &str) -> PResult<UCICommand> {
    "ucinewgame".value(UCICommand::UCINewGame).parse_next(input)
}

fn parse_position(input: &mut &str) -> PResult<UCICommand> {
    preceded(
        "position ",
        (
            alt((
                "startpos".value(None),
                preceded("fen ", parse_position_fen.map(Some)),
            )),
            opt(preceded(" moves ", rest)),
        )
            .try_map(|(fen, moves): (Option<String>, Option<&str>)| {
                Ok::<UCICommand, <Square as FromStr>::Err>(UCICommand::Position {
                    fen: fen.map(|s: String| s.to_string()),
                    moves: moves.map(|moves: &str| {
                        {
                            moves.split(' ').map(|mve_str| {
                                let (src_str, dest_str) = mve_str.split_at(2);
                                let src = Square::from_str(src_str.to_uppercase().as_str())?;
                                let dest = Square::from_str(dest_str.to_uppercase().as_str())?;
                                Ok::<Move, <Square as FromStr>::Err>(Move {
                                    src,
                                    dest,
                                    promotion: None,
                                })
                            })
                        }
                        .collect::<Result<Vec<Move>, _>>()
                        .unwrap()
                    }),
                })
            }),
    )
    .parse_next(input)
}

fn parse_position_fen(input: &mut &str) -> PResult<String> {
    (
        terminated(separated(8, alphanumeric1, '/'), ' '),
        terminated(one_of(['w', 'b']), ' '),
        terminated(take_while(0.., ('K', 'k', 'Q', 'q', '-')), ' '),
        terminated(alt((alphanumeric1, "-")), ' '),
        terminated(digit1, ' '),
        digit1,
    )
        .map(
            |(s1, s2, s3, s4, s5, s6): (Vec<&str>, char, &str, &str, &str, &str)| {
                let pieces_str = s1.join("/");
                format!("{} {} {} {} {} {}", pieces_str, s2, s3, s4, s5, s6)
            },
        )
        .parse_next(input)
}

fn parse_stop(input: &mut &str) -> PResult<UCICommand> {
    "stop".value(UCICommand::Stop).parse_next(input)
}

fn parse_ponderhit(input: &mut &str) -> PResult<UCICommand> {
    "ponderhit".value(UCICommand::PonderHit).parse_next(input)
}

fn parse_quit(input: &mut &str) -> PResult<UCICommand> {
    "quit".value(UCICommand::Quit).parse_next(input)
}

fn parse_eval(input: &mut &str) -> PResult<UCICommand> {
    "eval".value(UCICommand::Eval).parse_next(input)
}

fn parse_perft(input: &mut &str) -> PResult<UCICommand> {
    // We parse this separately than a GoParameter, even though it starts with `go`.
    // This is just to be consistent with stockfish
    preceded(
        "go perft ",
        digit1.try_map(|depth: &str| usize::from_str(depth)),
    )
    .map(|depth: usize| UCICommand::Perft { depth })
    .parse_next(input)
}

fn parse_perft_full(input: &mut &str) -> PResult<UCICommand> {
    // We parse this separately than a GoParameter, even though it starts with `go`.
    // This is just to be consistent with stockfish
    preceded(
        "go perft_full ",
        digit1.try_map(|depth: &str| usize::from_str(depth)),
    )
    .map(|depth: usize| UCICommand::PerftFull { depth })
    .parse_next(input)
}
fn parse_perft_benchmark(input: &mut &str) -> PResult<UCICommand> {
    "perft_bench"
        .value(UCICommand::PerftBenchmark)
        .parse_next(input)
}

// ======================================================
// Winnow Parsing functions (go commands)
// ======================================================

fn parse_go(input: &mut &str) -> PResult<UCICommand> {
    preceded(
        "go ",
        separated(
            1..,
            alt((
                parse_go_searchmoves,
                parse_go_ponder,
                parse_go_time,
                parse_go_inc,
                parse_go_movestogo,
                parse_go_depth,
                parse_go_nodes,
                parse_go_mate,
                parse_go_movetime,
                parse_go_infinite,
            )),
            ' ',
        ),
    )
    .map(|params: Vec<GoParameter>| SearchParams {
        search_moves: params.iter().find_map(|param| {
            if let GoParameter::SearchMoves { moves } = param {
                Some(moves.clone())
            } else {
                None
            }
        }),
        ponder: params.iter().any(|i| matches!(i, GoParameter::Ponder)),
        white_time: params.iter().find_map(|param| match param {
            GoParameter::Time {
                time,
                side: Side::White,
            } => Some(*time),
            _ => None,
        }),
        black_time: params.iter().find_map(|param| match param {
            GoParameter::Time {
                time,
                side: Side::Black,
            } => Some(*time),
            _ => None,
        }),
        white_inc: params.iter().find_map(|param| match param {
            GoParameter::Inc {
                time,
                side: Side::White,
            } => Some(*time),
            _ => None,
        }),
        black_inc: params.iter().find_map(|param| match param {
            GoParameter::Inc {
                time,
                side: Side::Black,
            } => Some(*time),
            _ => None,
        }),
        moves_to_go: params.iter().find_map(|param| {
            if let GoParameter::MovesToGo { moves } = param {
                Some(*moves)
            } else {
                None
            }
        }),
        max_depth: params.iter().find_map(|param| {
            if let GoParameter::Depth { moves } = param {
                Some(*moves)
            } else {
                None
            }
        }),
        max_nodes: params.iter().find_map(|param| {
            if let GoParameter::Nodes { nodes } = param {
                Some(*nodes)
            } else {
                None
            }
        }),
        mate: params.iter().find_map(|param| {
            if let GoParameter::Mate { moves } = param {
                Some(*moves)
            } else {
                None
            }
        }),
        move_time: params.iter().find_map(|param| {
            if let GoParameter::MoveTime { time } = param {
                Some(*time)
            } else {
                None
            }
        }),
        infinite: params.iter().any(|i| matches!(i, GoParameter::Infinite)),
        debug: false, // The UCI server stores state of `debug`, default to false
    })
    .map(|search_params: SearchParams| UCICommand::Go {
        params: search_params,
    })
    .parse_next(input)
}

fn parse_go_searchmoves(input: &mut &str) -> PResult<GoParameter> {
    preceded("searchmoves ", rest)
        .try_map(|moves: &str| {
            Ok::<GoParameter, <Square as FromStr>::Err>(GoParameter::SearchMoves {
                moves: moves
                    .split(' ')
                    .map(|mve_str| {
                        let (src_str, dest_str) = mve_str.split_at(2);
                        let src = Square::from_str(src_str.to_uppercase().as_str())?;
                        let dest = Square::from_str(dest_str.to_uppercase().as_str())?;
                        Ok::<Move, <Square as FromStr>::Err>(Move {
                            src,
                            dest,
                            promotion: None,
                        })
                    })
                    .collect::<Result<Vec<Move>, _>>()?,
            })
        })
        .parse_next(input)
}

fn parse_go_ponder(input: &mut &str) -> PResult<GoParameter> {
    "ponder".value(GoParameter::Ponder).parse_next(input)
}

fn parse_go_time(input: &mut &str) -> PResult<GoParameter> {
    alt((
        preceded("wtime ", digit1.try_map(|msec: &str| u64::from_str(msec)))
            .map(|msec: u64| Duration::from_millis(msec))
            .map(|time: Duration| GoParameter::Time {
                time,
                side: Side::White,
            }),
        preceded("btime ", digit1.try_map(|msec: &str| u64::from_str(msec)))
            .map(|msec: u64| Duration::from_millis(msec))
            .map(|time: Duration| GoParameter::Time {
                time,
                side: Side::Black,
            }),
    ))
    .parse_next(input)
}

fn parse_go_inc(input: &mut &str) -> PResult<GoParameter> {
    alt((
        preceded("winc ", digit1.try_map(|msec: &str| u64::from_str(msec)))
            .map(|msec: u64| Duration::from_millis(msec))
            .map(|time: Duration| GoParameter::Inc {
                time,
                side: Side::White,
            }),
        preceded("binc ", digit1.try_map(|msec: &str| u64::from_str(msec)))
            .map(|msec: u64| Duration::from_millis(msec))
            .map(|time: Duration| GoParameter::Inc {
                time,
                side: Side::Black,
            }),
    ))
    .parse_next(input)
}

fn parse_go_movestogo(input: &mut &str) -> PResult<GoParameter> {
    preceded(
        "movestogo ",
        digit1.try_map(|moves: &str| u16::from_str(moves)),
    )
    .map(|moves: u16| GoParameter::MovesToGo { moves })
    .parse_next(input)
}

fn parse_go_depth(input: &mut &str) -> PResult<GoParameter> {
    preceded("depth ", digit1.try_map(|moves: &str| u64::from_str(moves)))
        .map(|moves: u64| GoParameter::Depth { moves })
        .parse_next(input)
}

fn parse_go_nodes(input: &mut &str) -> PResult<GoParameter> {
    preceded("nodes ", digit1.try_map(|nodes: &str| u64::from_str(nodes)))
        .map(|nodes: u64| GoParameter::Nodes { nodes })
        .parse_next(input)
}

fn parse_go_mate(input: &mut &str) -> PResult<GoParameter> {
    preceded("mate ", digit1.try_map(|moves: &str| u64::from_str(moves)))
        .map(|moves: u64| GoParameter::Mate { moves })
        .parse_next(input)
}

fn parse_go_movetime(input: &mut &str) -> PResult<GoParameter> {
    preceded(
        "movetime ",
        digit1.try_map(|msec: &str| u64::from_str(msec)),
    )
    .map(|msec: u64| Duration::from_millis(msec))
    .map(|time: Duration| GoParameter::MoveTime { time })
    .parse_next(input)
}

fn parse_go_infinite(input: &mut &str) -> PResult<GoParameter> {
    "infinite".value(GoParameter::Infinite).parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;
    use testresult::TestResult;

    use engine::Square::*;

    #[test_case("uci", UCICommand::UCI)]
    #[test_case("debug on", UCICommand::Debug { on: true })]
    #[test_case("debug off", UCICommand::Debug { on: false })]
    #[test_case("isready", UCICommand::IsReady)]
    #[test_case("setoption name style value risky", UCICommand::SetOption { name: "style".to_string(), value: Some("risky".to_string()) })]
    #[test_case("setoption name multi word name value yes", UCICommand::SetOption { name: "multi word name".to_string(), value: Some("yes".to_string()) })]
    #[test_case("setoption name clear hash", UCICommand::SetOption { name: "clear hash".to_string(), value: None })]
    #[test_case("register name Will code 1234", UCICommand::Register { name: "Will".to_string(), code: "1234".to_string() })]
    #[test_case("register later", UCICommand::RegisterLater)]
    #[test_case("ucinewgame", UCICommand::UCINewGame)]
    #[test_case("position startpos moves e2e4 e7e5", UCICommand::Position { fen: None, moves: Some(vec![Move::new(E2, E4), Move::new(E7, E5)])} ; "position startpos moves e2e4 e7e5")]
    #[test_case("position fen 8/8/4Rp2/5P2/1PP1pkP1/7P/1P1r4/7K b - - 0 40", UCICommand::Position { fen: Some("8/8/4Rp2/5P2/1PP1pkP1/7P/1P1r4/7K b - - 0 40".to_string()), moves: None} ; "position fen 8/8/4Rp2/5P2/1PP1pkP1/7P/1P1r4/7K b - - 0 40")]
    #[test_case("position fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 moves f3f6", UCICommand::Position { fen: Some("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), moves: Some(vec![Move::new(F3, F6)])} ; "position fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 moves f3f6")]
    #[test_case("stop", UCICommand::Stop)]
    #[test_case("ponderhit", UCICommand::PonderHit)]
    #[test_case("quit", UCICommand::Quit)]
    #[test_case("go searchmoves e2e4 e7e5", UCICommand::Go { params: SearchParams{ search_moves: Some(vec![Move::new(Square::E2, Square::E4), Move::new(Square::E7, Square::E5)]), ..SearchParams::default()}} ; "go searchmoves e2e4 e7e5")]
    #[test_case("go ponder", UCICommand::Go { params: SearchParams { ponder: true, ..SearchParams::default() }} ; "go ponder")]
    #[test_case("go wtime 1000", UCICommand::Go { params: SearchParams { white_time: Some(Duration::from_millis(1000)), ..SearchParams::default() }} ; "go wtime 1000")]
    #[test_case("go btime 3", UCICommand::Go { params: SearchParams { black_time: Some(Duration::from_millis(3)), ..SearchParams::default() }} ; "go btime 3")]
    #[test_case("go winc 1000", UCICommand::Go { params: SearchParams { white_inc: Some(Duration::from_millis(1000)), ..SearchParams::default() }} ; "go winc 1000")]
    #[test_case("go binc 3", UCICommand::Go { params: SearchParams { black_inc: Some(Duration::from_millis(3)), ..SearchParams::default() }} ; "go binc 3 ")]
    #[test_case("go movestogo 7", UCICommand::Go { params: SearchParams { moves_to_go: Some(7), ..SearchParams::default() }} ; "go movestogo 7")]
    #[test_case("go depth 6", UCICommand::Go { params: SearchParams { max_depth: Some(6), ..SearchParams::default() }} ; "go depth 6")]
    #[test_case("go nodes 10000", UCICommand::Go { params: SearchParams { max_nodes: Some(10000), ..SearchParams::default() }} ; "go nodes 10000")]
    #[test_case("go mate 18", UCICommand::Go { params: SearchParams { mate: Some(18), ..SearchParams::default() }} ; "go mate 18")]
    #[test_case("go movetime 100", UCICommand::Go { params: SearchParams { move_time: Some(Duration::from_millis(100)), ..SearchParams::default() }} ; "go movetime 100")]
    #[test_case("go infinite", UCICommand::Go { params: SearchParams { infinite: true, ..SearchParams::default() }} ; "go infinite")]
    #[test_case("go infinite wtime 1000", UCICommand::Go { params: SearchParams { infinite: true, white_time: Some(Duration::from_millis(1000)), ..SearchParams::default() }} ; "go infinite wtime 1000")]
    #[test_case("go depth 10 searchmoves a2a4 b2b4", UCICommand::Go { params: SearchParams { max_depth: Some(10), search_moves: Some(vec![Move::new(Square::A2, Square::A4), Move::new(Square::B2, Square::B4)]), ..SearchParams::default() }} ; "go depth 10 searchmoves a2a4 b2b4")]
    fn test_from_str(input: &str, want: UCICommand) -> TestResult {
        let got = UCICommand::from_str(input)?;

        assert_eq!(got, want);
        Ok(())
    }

    #[test_case(
        "8/8/4Rp2/5P2/1PP1pkP1/7P/1P1r4/7K b - - 0 40".to_string(),
        "8/8/4Rp2/5P2/1PP1pkP1/7P/1P1r4/7K b - - 0 40".to_string()
    )]
    fn test_from_str_fen(input: String, want: String) -> TestResult {
        let got = parse_position_fen(&mut input.as_str())?;

        assert_eq!(got, want);
        Ok(())
    }
}
