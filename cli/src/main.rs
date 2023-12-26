use ::engine::bitboard::Square;
use ::engine::move_gen::{
    hyperbola_quintessence::HyperbolaQuintessence, leaping_pieces::LeapingPiecesMoveGen,
    AllPiecesMoveGen, GenerateAllMoves,
};
use ::engine::search::find_move;
use engine::evaluation::PositionEvaluator;
use engine::position::{Move, Position, PositionError};
use std::{io, str::FromStr};

fn play_game() -> Result<(), PositionError> {
    let move_gen = AllPiecesMoveGen::new(
        Box::new(LeapingPiecesMoveGen::new()),
        Box::new(HyperbolaQuintessence::new()),
    );
    let position_evaluator = PositionEvaluator {};

    let mut position = Position::start();

    println!("{:?}", position);
    loop {
        let moves = move_gen.gen_moves(&position);
        if moves.len() == 0 {
            break;
        }

        let mut move_buffer = String::new();
        io::stdin().read_line(&mut move_buffer).unwrap();

        if move_buffer.len() != 6 {
            println!("input: <SQUARE>,<SQUARE>");
            continue;
        }

        let src_string = &move_buffer[..2];
        let src = Square::from_str(src_string).unwrap();
        let dest_string = &move_buffer[3..5];
        let dest = Square::from_str(dest_string).unwrap();

        let mve = Move::new(src, dest);

        if !moves.contains(&mve) {
            continue;
        }

        position.make_move(&mve)?;
        println!("{:?}", position);
        let mve = find_move(&position, &position_evaluator, &move_gen, 4);
        position.make_move(&mve)?;
        println!("{:?}", position);
    }
    Ok(())
}

fn main() {
    play_game().unwrap();
}
