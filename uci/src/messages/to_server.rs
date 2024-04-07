use std::str::FromStr;

use engine::bitboard::Square;
use winnow::ascii::{alpha1, digit1, till_line_ending};
use winnow::combinator::{alt, empty, opt, preceded, repeat_till, rest, separated, seq};
use winnow::token::{take_till, take_until};
use winnow::{PResult, Parser};

use engine::position::{Move, Side};

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum UCIMessageToServer {
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
    MoveTime { msec: u64 },
    Infinite,
    Ponder,
}

impl FromStr for UCIMessageToServer {
    type Err = String;
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
        .map_err(|_| format!("cannot parse: [{}]", input))
    }
}

// Winnow parse functions
fn parse_uci(input: &mut &str) -> PResult<UCIMessageToServer> {
    println!("{}", input);
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
    println!("{}", input);
    "ucinewgame"
        .value(UCIMessageToServer::UCINewGame)
        .parse_next(input)
}

fn parse_position(input: &mut &str) -> PResult<UCIMessageToServer> {
    preceded(
        "position ",
        (
            alt((
                "startpos".value(None),
                take_until(0.., " moves").map(|fen| Some(fen)),
            )),
            preceded(" moves ", rest),
        )
            .map(
                |(fen, moves): (Option<&str>, &str)| UCIMessageToServer::Position {
                    fen: fen.map(|s: &str| s.to_string()),
                    moves: moves
                        .split(' ')
                        .map(|mve_str| {
                            let (src_str, dest_str) = mve_str.split_at(2);
                            Move {
                                src: Square::from_str(src_str.to_uppercase().as_str())
                                    .expect(format!("couldn't parse string {}", src_str).as_str()),
                                dest: Square::from_str(dest_str.to_uppercase().as_str())
                                    .expect(format!("couldn't parse string {}", dest_str).as_str()),
                                promotion: None,
                            }
                        })
                        .collect(),
                },
            ),
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

fn parse_go(input: &mut &str) -> PResult<UCIMessageToServer> {
    preceded(
        "go ",
        separated(
            0..,
            alt((parse_go_searchmoves, parse_go_infinite, parse_go_time)),
            opt(' '),
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
    #[test_case("go infinite", UCIMessageToServer::Go { params: vec![GoParameter::Infinite]})]
    fn test_from_str(input: &str, want: UCIMessageToServer) -> TestResult {
        let got = UCIMessageToServer::from_str(input)?;

        assert_eq!(got, want);
        Ok(())
    }
}
