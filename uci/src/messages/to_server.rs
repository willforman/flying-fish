use std::fmt::Display;
use std::str::FromStr;

use engine::bitboard::Square;
use winnow::ascii::digit1;
use winnow::combinator::{alt, preceded, rest, separated};
use winnow::token::take_until;
use winnow::{PResult, Parser};

use engine::position::{Move, Side};

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum UCIMessageToServer {
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
        moves: Vec<Move>,
    },
    Go {
        params: Vec<GoParameter>,
    },
    Stop,
    PonderHit,
    Quit,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum GoParameter {
    SearchMoves { moves: Vec<Move> },
    Time { msec: u64, side: Side },
    Inc { msec: u64, side: Side },
    MovesToGo { moves: u64 },
    Depth { moves: u64 },
    Nodes { nodes: u64 },
    Mate { moves: u64 },
    MoveTime { msec: u64 },
    Infinite,
    Ponder,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct UCIMessageToServerParseError(String);

impl Display for UCIMessageToServerParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for UCIMessageToServerParseError {}

impl FromStr for UCIMessageToServer {
    type Err = UCIMessageToServerParseError;
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
            parse_go,
        ))
        .parse(input)
        .map_err(|_| UCIMessageToServerParseError(format!("cannot parse: [{}]", input)))
    }
}

// ======================================================
// Winnow Parsing functions (non go commands)
// ======================================================

fn parse_uci(input: &mut &str) -> PResult<UCIMessageToServer> {
    "uci".parse_next(input).map(|_| UCIMessageToServer::UCI)
}

fn parse_debug(input: &mut &str) -> PResult<UCIMessageToServer> {
    preceded("debug ", alt(("on".value(true), "off".value(false))))
        .parse_next(input)
        .map(|on| UCIMessageToServer::Debug { on })
}

fn parse_isready(input: &mut &str) -> PResult<UCIMessageToServer> {
    "isready"
        .parse_next(input)
        .map(|_| UCIMessageToServer::IsReady)
}

fn parse_setoption(input: &mut &str) -> PResult<UCIMessageToServer> {
    preceded(
        "setoption name ",
        alt((
            (take_until(0.., " value"), preceded(" value ", rest)).map(
                |(name, value): (&str, &str)| UCIMessageToServer::SetOption {
                    name: name.to_string(),
                    value: Some(value.to_string()),
                },
            ),
            rest.map(|name: &str| UCIMessageToServer::SetOption {
                name: name.to_string(),
                value: None,
            }),
        )),
    )
    .parse_next(input)
}

fn parse_register(input: &mut &str) -> PResult<UCIMessageToServer> {
    preceded(
        "register name ",
        (take_until(0.., " code"), preceded(" code ", rest)).map(|(name, code): (&str, &str)| {
            UCIMessageToServer::Register {
                name: name.to_string(),
                code: code.to_string(),
            }
        }),
    )
    .parse_next(input)
}

fn parse_register_later(input: &mut &str) -> PResult<UCIMessageToServer> {
    "register later"
        .value(UCIMessageToServer::RegisterLater)
        .parse_next(input)
}

fn parse_ucinewgame(input: &mut &str) -> PResult<UCIMessageToServer> {
    "ucinewgame"
        .value(UCIMessageToServer::UCINewGame)
        .parse_next(input)
}

fn parse_position(input: &mut &str) -> PResult<UCIMessageToServer> {
    preceded(
        "position ",
        (
            alt(("startpos".value(None), take_until(0.., " moves").map(Some))),
            preceded(" moves ", rest),
        )
            .try_map(|(fen, moves): (Option<&str>, &str)| {
                Ok::<UCIMessageToServer, <Square as FromStr>::Err>(UCIMessageToServer::Position {
                    fen: fen.map(|s: &str| s.to_string()),
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
            }),
    )
    .parse_next(input)
}

fn parse_stop(input: &mut &str) -> PResult<UCIMessageToServer> {
    "stop".value(UCIMessageToServer::Stop).parse_next(input)
}

fn parse_ponderhit(input: &mut &str) -> PResult<UCIMessageToServer> {
    "ponderhit"
        .value(UCIMessageToServer::PonderHit)
        .parse_next(input)
}

fn parse_quit(input: &mut &str) -> PResult<UCIMessageToServer> {
    "quit".value(UCIMessageToServer::Quit).parse_next(input)
}

// ======================================================
// Winnow Parsing functions (go commands)
// ======================================================

fn parse_go(input: &mut &str) -> PResult<UCIMessageToServer> {
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
    .map(|params: Vec<GoParameter>| UCIMessageToServer::Go { params })
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
        preceded("wtime ", digit1.try_map(|msec: &str| u64::from_str(msec))).map(|msec: u64| {
            GoParameter::Time {
                msec,
                side: Side::White,
            }
        }),
        preceded("btime ", digit1.try_map(|msec: &str| u64::from_str(msec))).map(|msec: u64| {
            GoParameter::Time {
                msec,
                side: Side::Black,
            }
        }),
    ))
    .parse_next(input)
}

fn parse_go_inc(input: &mut &str) -> PResult<GoParameter> {
    alt((
        preceded("winc ", digit1.try_map(|msec: &str| u64::from_str(msec))).map(|msec: u64| {
            GoParameter::Inc {
                msec,
                side: Side::White,
            }
        }),
        preceded("binc ", digit1.try_map(|msec: &str| u64::from_str(msec))).map(|msec: u64| {
            GoParameter::Inc {
                msec,
                side: Side::Black,
            }
        }),
    ))
    .parse_next(input)
}

fn parse_go_movestogo(input: &mut &str) -> PResult<GoParameter> {
    preceded(
        "movestogo ",
        digit1.try_map(|moves: &str| u64::from_str(moves)),
    )
    .map(|moves: u64| GoParameter::MovesToGo { moves })
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
    .map(|msec: u64| GoParameter::MoveTime { msec })
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

    use engine::bitboard::Square::*;

    #[test_case("uci", UCIMessageToServer::UCI)]
    #[test_case("debug on", UCIMessageToServer::Debug { on: true })]
    #[test_case("debug off", UCIMessageToServer::Debug { on: false })]
    #[test_case("isready", UCIMessageToServer::IsReady)]
    #[test_case("setoption name style value risky", UCIMessageToServer::SetOption { name: "style".to_string(), value: Some("risky".to_string()) })]
    #[test_case("setoption name multi word name value yes", UCIMessageToServer::SetOption { name: "multi word name".to_string(), value: Some("yes".to_string()) })]
    #[test_case("setoption name clear hash", UCIMessageToServer::SetOption { name: "clear hash".to_string(), value: None })]
    #[test_case("register name Will code 1234", UCIMessageToServer::Register { name: "Will".to_string(), code: "1234".to_string() })]
    #[test_case("register later", UCIMessageToServer::RegisterLater)]
    #[test_case("ucinewgame", UCIMessageToServer::UCINewGame)]
    #[test_case("position startpos moves e2e4 e7e5", UCIMessageToServer::Position { fen: None, moves: vec![Move::new(E2, E4), Move::new(E7, E5)]})]
    #[test_case("position r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 moves f3f6", UCIMessageToServer::Position { fen: Some("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string()), moves: vec![Move::new(F3, F6)]})]
    #[test_case("stop", UCIMessageToServer::Stop)]
    #[test_case("ponderhit", UCIMessageToServer::PonderHit)]
    #[test_case("quit", UCIMessageToServer::Quit)]
    #[test_case("go searchmoves e2e4 e7e5", UCIMessageToServer::Go { params: vec![GoParameter::SearchMoves { moves: vec![Move::new(E2, E4), Move::new(E7, E5)]}]})]
    #[test_case("go ponder", UCIMessageToServer::Go { params: vec![GoParameter::Ponder]})]
    #[test_case("go wtime 1000", UCIMessageToServer::Go { params: vec![GoParameter::Time { msec: 1000, side: Side::White }]})]
    #[test_case("go btime 3", UCIMessageToServer::Go { params: vec![GoParameter::Time { msec: 3, side: Side::Black }]})]
    #[test_case("go winc 1000", UCIMessageToServer::Go { params: vec![GoParameter::Inc { msec: 1000, side: Side::White }]})]
    #[test_case("go binc 3", UCIMessageToServer::Go { params: vec![GoParameter::Inc { msec: 3, side: Side::Black }]})]
    #[test_case("go movestogo 7", UCIMessageToServer::Go { params: vec![GoParameter::MovesToGo { moves: 7}]})]
    #[test_case("go depth 6", UCIMessageToServer::Go { params: vec![GoParameter::Depth { moves: 6}]})]
    #[test_case("go nodes 10000", UCIMessageToServer::Go { params: vec![GoParameter::Nodes { nodes: 10000}]})]
    #[test_case("go mate 18", UCIMessageToServer::Go { params: vec![GoParameter::Mate { moves: 18}]})]
    #[test_case("go movetime 100", UCIMessageToServer::Go { params: vec![GoParameter::MoveTime { msec: 100}]})]
    #[test_case("go infinite", UCIMessageToServer::Go { params: vec![GoParameter::Infinite]})]
    #[test_case("go infinite wtime 1000", UCIMessageToServer::Go { params: vec![GoParameter::Infinite, GoParameter::Time { msec: 1000, side: Side::White }]})]
    #[test_case("go depth 10 searchmoves a2a4 b2b4", UCIMessageToServer::Go { params: vec![GoParameter::Depth { moves: 10 }, GoParameter::SearchMoves { moves: vec![Move::new(A2, A4), Move::new(B2, B4)] }]})]
    fn test_from_str(input: &str, want: UCIMessageToServer) -> TestResult {
        let got = UCIMessageToServer::from_str(input)?;

        assert_eq!(got, want);
        Ok(())
    }
}
