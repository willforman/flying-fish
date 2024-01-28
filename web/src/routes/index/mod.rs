use leptos::*;

pub mod chess_board;
pub mod moves;

use engine::move_gen::{HYPERBOLA_QUINTESSENCE_MOVE_GEN, HyperbolaQuintessenceMoveGen, GenerateMoves};
use engine::evaluation::POSITION_EVALUATOR;
use engine::position::{Move, Position, Side};
use engine::bitboard::Square;
use engine::search::search;

use crate::routes::index::chess_board::ChessBoard;
use crate::routes::index::moves::Moves;

const SEARCH_DEPTH: u32 = 3;

static MOVE_GEN: HyperbolaQuintessenceMoveGen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;

#[server(GenerateMove)]
async fn generate_move(position: Position, depth: u32) -> Result<Option<Move>, ServerFnError> {
    let moves = MOVE_GEN.gen_moves(&position);

    if moves.is_empty() {
        return Ok(None);
    }

    let mut best_val = if position.state.to_move == Side::White {
        f64::MIN
    } else {
        f64::MAX
    };

    let mut best_move: Option<Move> = None;

    for mve in moves {
        let mut move_position = position.clone();
        move_position.make_move(&mve)?;

        let got_val = search(&move_position, depth, MOVE_GEN, POSITION_EVALUATOR);

        if position.state.to_move == Side::White {
            if got_val > best_val {
                best_val = got_val;
                best_move = Some(mve);
            }
        } else {
            if got_val < best_val {
                best_val = got_val;
                best_move = Some(mve);
            }
        }
    }

    Ok(best_move)
}

#[component]
pub fn IndexPage() -> impl IntoView {
    let (game_complete, set_game_complete) = create_signal(false);
    let (position, set_position) = create_signal(Position::start());
    let (side, set_side) = create_signal(Side::White);
    let (moves, set_moves) = create_signal(Vec::<Move>::new());

    let handle_move = create_action(move |input: &Move| {
        set_position.update(|pos| pos.make_move(&input).unwrap() );
        set_moves.update(|moves| moves.push(input.clone()));
        async move {
            let maybe_generated_move = generate_move(position(), SEARCH_DEPTH).await.unwrap();
            if let Some(generated_move) = maybe_generated_move {
                set_position.update(|pos| pos.make_move(&generated_move).unwrap());
                set_moves.update(|moves| moves.push(generated_move.to_owned()));
            } else {
                set_game_complete(true);
            }
        }
    });

    let game_title = move || game_complete().then(|| "Game over");

    view! {
        <div class="flex">
            <div class="flex-initial">
                <Moves moves={moves} />
            </div>
            <div class="flex-1 justify-center mx-8">
                <h1 class="text-xl">
                    {game_title}
                </h1>
                <ChessBoard position=position player_side=side handle_move={handle_move} />
            </div>
            <div class="flex-initial bg-gray-200 p-2">
                <h3 class="text-xl font-bold">"move generation"</h3>
            </div>
        </div>
    }
}
