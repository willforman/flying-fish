use leptos::*;

pub mod chess_board;
pub mod moves;

use engine::algebraic_notation::move_to_algebraic_notation;
use engine::bitboard::Square;
use engine::evaluation::POSITION_EVALUATOR;
use engine::move_gen::{
    GenerateMoves, HyperbolaQuintessenceMoveGen, HYPERBOLA_QUINTESSENCE_MOVE_GEN,
};
use engine::position::{Move, Position, Side};
use engine::search::search;
use leptos::html;
use leptos::logging::log;
use web_sys::{Event, SubmitEvent};

use crate::routes::index::chess_board::ChessBoard;
use crate::routes::index::moves::Moves;

const SEARCH_DEPTH: u32 = 3;

static MOVE_GEN: HyperbolaQuintessenceMoveGen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;

#[server(GenerateMove)]
async fn generate_move(position: Position, depth: u32) -> Result<Option<Move>, ServerFnError> {
    let best_move = search(&position, depth, MOVE_GEN, POSITION_EVALUATOR);
    Ok(best_move)
}

#[component]
pub fn IndexPage() -> impl IntoView {
    let (game_complete, set_game_complete) = create_signal(false);
    let (position, set_position) = create_signal(Position::start());
    let (side, set_side) = create_signal(Side::White);
    let (move_strs, set_move_strs) = create_signal(Vec::<String>::new());

    let handle_move = create_action(move |input: &Move| {
        let move_str = move_to_algebraic_notation(&position(), input, MOVE_GEN, MOVE_GEN).unwrap();
        set_move_strs.update(|move_strs| move_strs.push(move_str));

        set_position.update(|pos| pos.make_move(&input).unwrap());

        async move {
            let maybe_generated_move = generate_move(position(), SEARCH_DEPTH).await.unwrap();
            if let Some(generated_move) = maybe_generated_move {
                let move_str =
                    move_to_algebraic_notation(&position(), &generated_move, MOVE_GEN, MOVE_GEN)
                        .unwrap();
                set_move_strs.update(|move_strs| move_strs.push(move_str));

                set_position.update(|pos| pos.make_move(&generated_move).unwrap());
            } else {
                set_game_complete(true);
            }
        }
    });

    let fen_input_element: NodeRef<html::Input> = create_node_ref();

    let handle_fen_submit = move |event: SubmitEvent| {
        event.prevent_default();

        let fen = fen_input_element()
            .expect("<input> should be mounted")
            .value();

        if let Ok(pos) = Position::from_fen(&fen) {
            set_position(pos);
        } else {
            log!("invalid fen given {fen}");
        }
    };

    let game_title = move || game_complete().then(|| "Game over");

    view! {
        <div class="flex items-start">
            <div class="flex-initial h-[40rem] w-64">
                <Moves move_strs={move_strs} />
            </div>
            <div class="flex-initial justify-center mx-8">
                <h1 class="text-xl">
                    {game_title}
                </h1>
                <form on:submit=handle_fen_submit>
                    <input type="text" node_ref=fen_input_element />
                    <input type="submit" value="Submit FEN"/>
                </form>
                <ChessBoard position=position player_side=side handle_move={handle_move} />
            </div>
        </div>
    }
}
