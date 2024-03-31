use engine::position::Move;

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

impl TryFrom<String> for UCIMessageToServer {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "uci" => Ok(Self::UCI),
            "debug on" => Ok(Self::Debug { on: true }),
            "debug off" => Ok(Self::Debug { on: false }),
            "isready" => Ok(Self::IsReady),

            _ => Err("Invalid pattern"),
        }
    }
}
