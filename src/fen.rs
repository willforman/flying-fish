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
    if castling_rights_str.is_empty() || castling_rights_str == "-" {
        return Ok(CastlingRights::new(false, false, false, false));
    }

    let mut white_king_side = false;
    let mut white_queen_side = false;
    let mut black_king_side = false;
    let mut black_queen_side = false;

    for (idx, ch) in castling_rights_str.chars().enumerate() {
        match ch {
            'K' => {
                if white_king_side {
                    return Err(FenParseError::CastlingRights(castling_rights_str.to_string(), idx))
                }
                white_king_side = true;
            }
            'Q' => {
                if white_queen_side {
                    return Err(FenParseError::CastlingRights(castling_rights_str.to_string(), idx));
                }
                white_queen_side = true;
            }
            'k' => {
                if black_king_side {
                    return Err(FenParseError::CastlingRights(castling_rights_str.to_string(), idx));
                }
                black_king_side = true;
            }
            'q' => {
                if black_queen_side {
                    return Err(FenParseError::CastlingRights(castling_rights_str.to_string(), idx));
                }
                black_queen_side = true;
            }
            _ => return Err(FenParseError::CastlingRights(castling_rights_str.to_string(), idx)),
        }
    }

    Ok(CastlingRights::new(white_king_side, white_queen_side, black_king_side, black_queen_side))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("-", CastlingRights::new(false, false, false, false) ; "empty")]
    #[test_case("KQkq", CastlingRights::new(true, true, true, true)  ; "KQkq")]
    #[test_case("Qk", CastlingRights::new(false, true, true, false)  ; "Qk")]
    #[test_case("K", CastlingRights::new(true, false, false, false)  ; "K")]
    fn test_castling_rights_from_fen(inp: &str, want: CastlingRights) {
        let got = castling_rights_from_fen(inp);
        assert!(got.is_ok());
        assert_eq!(got.unwrap(), want);
    }

    #[test_case("abc")]
    fn test_castling_rights_from_fen_invalid(inp: &str) {
        let got = castling_rights_from_fen(inp);
        assert!(matches!(got, Err(FenParseError::CastlingRights(_, _))));
    }
}
