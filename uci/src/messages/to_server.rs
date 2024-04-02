use std::str::FromStr;

use winnow::combinator::{alt, preceded, seq};
use winnow::{PResult, Parser};

use engine::position::Move;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum UCIMessageToServer {
    UCI,
    Debug { on: bool },
    IsReady,
    SetOption { name: String, value: Option<String> },
    UCINewGame,
    Position { fen: String, moves: Vec<Move> },
    Go,
    Stop,
    PonderHit,
    Quit,
}

impl FromStr for UCIMessageToServer {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        alt((parse_uci, parse_debug, parse_isready))
            .parse(input)
            .map_err(|_| format!("cannot parse {}", input))
    }
}

// Winnow parse functions
fn parse_uci(input: &mut &str) -> PResult<UCIMessageToServer> {
    "uci".parse_next(input).map(|_| UCIMessageToServer::UCI)
}

fn parse_debug(input: &mut &str) -> PResult<UCIMessageToServer> {
    let on = preceded("debug ", alt(("on".value(true), "off".value(false)))).parse_next(input)?;

    Ok(UCIMessageToServer::Debug { on })
}

fn parse_isready(input: &mut &str) -> PResult<UCIMessageToServer> {
    "isready"
        .parse_next(input)
        .map(|_| UCIMessageToServer::IsReady)
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;
    use testresult::TestResult;

    #[test_case("uci", UCIMessageToServer::UCI)]
    #[test_case("debug on", UCIMessageToServer::Debug { on: true })]
    #[test_case("debug off", UCIMessageToServer::Debug { on: false })]
    #[test_case("isready", UCIMessageToServer::IsReady)]
    fn test_from_str(input: &str, want: UCIMessageToServer) -> TestResult {
        let got = UCIMessageToServer::from_str(input)?;

        assert_eq!(got, want);
        Ok(())
    }
}
