use crate::bitboard::Square::*;
use crate::bitboard::{BitBoard, Square};
use crate::position::zobrist_hash::ZobristHash;
use crate::position::{CastlingRights, Piece, Position, Side, State};
use std::str::FromStr;

#[derive(thiserror::Error, Debug)]
pub enum FenParseError {
    #[error("num fields: want 6 got {0}")]
    NumFields(usize),

    #[error("piece placement: got {0}, err at {1}")]
    PiecePlacement(String, usize),

    #[error("side to move: want 'w'|'b' got {0}")]
    SideToMove(String),

    #[error("castling rights given: got {0}, err at idx {1}")]
    CastlingRights(String, usize),

    #[error("en passant target: got {0}")]
    EnPassantTarget(String),

    #[error("halfmove clock: want 0 <= x < 50 got {0}")]
    HalfmoveClock(String),

    #[error("full move counter: want 0 <= x < 65_535 got {0}")]
    FullMoveCounter(String),
}

impl Position {
    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        let fields = fen.split(' ').collect::<Vec<&str>>();

        if fields.len() != 6 {
            Err(FenParseError::NumFields(fields.len()))?
        }

        let (sides, pieces) = pieces_from_fen(fields[0])?;

        let to_move = match fields[1] {
            "w" => Side::White,
            "b" => Side::Black,
            _ => Err(FenParseError::SideToMove(String::from(fields[1])))?,
        };

        let half_move_clock = fields[4]
            .parse::<u8>()
            .map_err(|_| FenParseError::HalfmoveClock(fields[4].to_string()))?;

        // Half move counter must be in 0..=49
        // Don't have to check if less than zero because u8 min value = 0
        if half_move_clock >= 50 {
            Err(FenParseError::HalfmoveClock(fields[4].to_string()))?
        }

        let full_move_counter = fields[5]
            .parse()
            .map_err(|_| FenParseError::FullMoveCounter(fields[5].to_string()))?;

        let state = State {
            castling_rights: castling_rights_from_fen(fields[2])?,
            en_passant_target: en_passant_target_from_fen(fields[3])?,
            half_move_clock,
            to_move,
            full_move_counter,
        };

        let zobrist_hash = ZobristHash::calculate(&pieces, &state);

        Ok(Position {
            sides,
            pieces,
            state,
            zobrist_hash,
        })
    }

    pub fn to_fen(&self) -> String {
        let mut pieces = String::with_capacity(64);
        let mut curr_empty_count = 0;

        for (idx, &sq) in Square::list_black_perspective().iter().rev().enumerate() {
            if let Some((piece, side)) = self.is_piece_at_no_side(sq) {
                if curr_empty_count != 0 {
                    pieces += &curr_empty_count.to_string();
                    curr_empty_count = 0;
                }
                let piece_char: char = if side == Side::White {
                    <Piece as Into<char>>::into(piece).to_ascii_uppercase()
                } else {
                    <Piece as Into<char>>::into(piece)
                };
                pieces += &piece_char.to_string();
            } else {
                curr_empty_count += 1;
            }
            if (idx + 1) % 8 == 0 {
                if curr_empty_count != 0 {
                    pieces += &curr_empty_count.to_string();
                    curr_empty_count = 0;
                }
                if idx != 63 {
                    pieces += "/";
                }
            }
        }

        let side_to_move_char = if self.state.to_move == Side::White {
            'w'
        } else {
            'b'
        };

        let mut castling_rights = String::with_capacity(4);

        if self.state.castling_rights.white_king_side {
            castling_rights += "K";
        }
        if self.state.castling_rights.white_queen_side {
            castling_rights += "Q";
        }
        if self.state.castling_rights.black_king_side {
            castling_rights += "k";
        }
        if self.state.castling_rights.black_queen_side {
            castling_rights += "q";
        }

        if castling_rights.is_empty() {
            castling_rights += "-";
        }

        let en_passant = if let Some(ep_target) = self.state.en_passant_target {
            ep_target.to_string().to_ascii_lowercase()
        } else {
            "-".to_string()
        };

        format!(
            "{} {} {} {} {} {}",
            pieces, side_to_move_char, castling_rights, en_passant, self.state.half_move_clock, 1
        )
    }
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
                    return Err(FenParseError::CastlingRights(
                        castling_rights_str.to_string(),
                        idx,
                    ));
                }
                white_king_side = true;
            }
            'Q' => {
                if white_queen_side {
                    return Err(FenParseError::CastlingRights(
                        castling_rights_str.to_string(),
                        idx,
                    ));
                }
                white_queen_side = true;
            }
            'k' => {
                if black_king_side {
                    return Err(FenParseError::CastlingRights(
                        castling_rights_str.to_string(),
                        idx,
                    ));
                }
                black_king_side = true;
            }
            'q' => {
                if black_queen_side {
                    return Err(FenParseError::CastlingRights(
                        castling_rights_str.to_string(),
                        idx,
                    ));
                }
                black_queen_side = true;
            }
            _ => {
                return Err(FenParseError::CastlingRights(
                    castling_rights_str.to_string(),
                    idx,
                ));
            }
        }
    }

    Ok(CastlingRights::new(
        white_king_side,
        white_queen_side,
        black_king_side,
        black_queen_side,
    ))
}

fn en_passant_target_from_fen(
    en_passant_target_str: &str,
) -> Result<Option<Square>, FenParseError> {
    if en_passant_target_str == "-" {
        return Ok(None);
    }

    // FEN uses lowercase letter for square names, Square uses uppercase
    Square::from_str(&en_passant_target_str.to_uppercase())
        .map_err(|_| FenParseError::EnPassantTarget(en_passant_target_str.to_string()))
        .map(Some)
}

const FEN_SQUARE_ORDER: [Square; 64] = [
    A8, B8, C8, D8, E8, F8, G8, H8, A7, B7, C7, D7, E7, F7, G7, H7, A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5, A4, B4, C4, D4, E4, F4, G4, H4, A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2, A1, B1, C1, D1, E1, F1, G1, H1,
];

fn pieces_from_fen(pieces_str: &str) -> Result<([BitBoard; 2], [BitBoard; 12]), FenParseError> {
    let mut sides = [BitBoard::empty(); 2];
    let mut pieces = [BitBoard::empty(); 12];
    let mut sq_idx = 0;

    for (ch_idx, ch) in pieces_str.chars().enumerate() {
        if let Ok(piece) = Piece::try_from(ch.to_ascii_lowercase()) {
            let square = FEN_SQUARE_ORDER[sq_idx];
            let side = if ch.is_uppercase() {
                Side::White
            } else {
                Side::Black
            };

            sides[side as usize].set_square(square);
            pieces[piece as usize + (side as usize * 6)].set_square(square);

            sq_idx += 1;
        } else if let Some(digit) = ch.to_digit(10) {
            sq_idx += digit as usize;
        } else if ch == '/' {
            // pass
        } else {
            Err(FenParseError::PiecePlacement(
                pieces_str.to_string(),
                ch_idx,
            ))?
        }
    }

    Ok((sides, pieces))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    use testresult::TestResult;

    #[test_case("-", CastlingRights::new(false, false, false, false) ; "empty")]
    #[test_case("KQkq", CastlingRights::new(true, true, true, true)  ; "KQkq")]
    #[test_case("Qk", CastlingRights::new(false, true, true, false)  ; "Qk")]
    #[test_case("K", CastlingRights::new(true, false, false, false)  ; "K")]
    fn test_castling_rights_from_fen(inp: &str, want: CastlingRights) -> TestResult {
        let got = castling_rights_from_fen(inp)?;
        assert_eq!(got, want);
        Ok(())
    }

    #[test_case("abc")]
    fn test_castling_rights_from_fen_invalid(inp: &str) {
        let got = castling_rights_from_fen(inp);
        assert!(matches!(got, Err(FenParseError::CastlingRights(_, _))));
    }

    #[test_case("-", None      ; "empty")]
    #[test_case("e3", Some(E3) ; "e3")]
    #[test_case("c6", Some(C6) ; "c6")]
    fn test_en_passant_target_from_fen(inp: &str, want: Option<Square>) -> TestResult {
        let got = en_passant_target_from_fen(inp)?;
        assert_eq!(got, want);
        Ok(())
    }

    #[test_case("abc")]
    fn test_en_passant_target_from_fen_invalid(inp: &str) {
        let got = en_passant_target_from_fen(inp);
        assert!(matches!(got, Err(FenParseError::EnPassantTarget(_))));
    }

    // 1R2k3/2Q5/8/8/7p/8/5P1P/6K1 b - - 7 42
    #[test_case("1R2k3/2Q5/8/8/7p/8/5P1P/6K1", [
        BitBoard::from_squares(&[B8, C7, F2, G1, H2]),
        BitBoard::from_squares(&[E8, H4])
    ], [
            BitBoard::from_squares(&[F2, H2]),
            BitBoard::from_squares(&[]),
            BitBoard::from_squares(&[]),
            BitBoard::from_squares(&[B8]),
            BitBoard::from_squares(&[C7]),
            BitBoard::from_squares(&[G1]),
            BitBoard::from_squares(&[H4]),
            BitBoard::from_squares(&[]),
            BitBoard::from_squares(&[]),
            BitBoard::from_squares(&[]),
            BitBoard::from_squares(&[]),
            BitBoard::from_squares(&[E8]),
    ] ; "first")]
    fn test_pieces_from_fen(
        inp: &str,
        sides_want: [BitBoard; 2],
        pieces_want: [BitBoard; 12],
    ) -> TestResult {
        let (sides, pieces) = pieces_from_fen(inp)?;

        assert_eq!(sides[0], sides_want[0]);
        assert_eq!(sides[1], sides_want[1]);

        assert_eq!(pieces[0], pieces_want[0]);
        assert_eq!(pieces[1], pieces_want[1]);
        assert_eq!(pieces[2], pieces_want[2]);
        assert_eq!(pieces[3], pieces_want[3]);
        assert_eq!(pieces[4], pieces_want[4]);
        assert_eq!(pieces[5], pieces_want[5]);
        assert_eq!(pieces[6], pieces_want[6]);
        assert_eq!(pieces[7], pieces_want[7]);
        assert_eq!(pieces[8], pieces_want[8]);
        assert_eq!(pieces[9], pieces_want[9]);
        assert_eq!(pieces[10], pieces_want[10]);
        assert_eq!(pieces[11], pieces_want[11]);

        Ok(())
    }

    #[test_case(
        Position::start(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string() ; "starting position"
    )]
    fn test_to_fen_position(position: Position, want: String) {
        let got = position.to_fen();
        assert_eq!(got, want);
    }

    #[test_case(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1" ; "kiwipete"
    )]
    #[test_case(
        "8/8/8/4k3/8/3P4/5K2/r7 w - - 1 1" ; "random"
    )]
    fn test_to_fen_string(fen: &str) -> TestResult {
        let pos = Position::from_fen(fen)?;
        let got = pos.to_fen();
        assert_eq!(got, fen);
        Ok(())
    }
}
