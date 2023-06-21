use crate::position::{Position,Side,CastlingRights};

#[derive(thiserror::Error, Debug)]
pub enum FenParseError {
    #[error("num fields: want 6 got {0}")]
    NumFields(usize),

    #[error("side to move: want 'w'|'b' got {0}")]
    SideToMove(String),

    #[error("castling rights given: got {0}, err at idx {1}")]
    CastlingRights(String, usize),
}

pub fn from_fen(fen: &str) -> Result<Position, FenParseError> {
    let fields = fen.split(' ').collect::<Vec<&str>>(); 

    if fields.len() != 6 {
        Err(FenParseError::NumFields(fields.len()))?
    }

    let to_move = match fields[1] {
        "w" => Side::White,
        "b" => Side::Black,
        _ => Err(FenParseError::SideToMove(String::from(fields[1])))?
    };

    Ok(Position::start())
}

fn castling_rights_from_fen(castling_rights_str: &str) -> Result<CastlingRights, FenParseError> {
    if castling_rights_str.len() == 0 {
        FenParseError::CastlingRights("", 0)?
    }

    Ok(CastlingRights)

}
