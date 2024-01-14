use leptos::*;

pub mod chess_board;

use crate::routes::index::chess_board::ChessBoard;
use engine::move_gen::{HYPERBOLA_QUINTESSENCE_MOVE_GEN, HyperbolaQuintessenceMoveGen};
use engine::evaluation::POSITION_EVALUATOR;
use engine::position::{Move, Position, Side};
use engine::bitboard::Square;
use engine::search::find_move;

static MOVE_GEN: HyperbolaQuintessenceMoveGen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;

#[server(GenerateMove)]
async fn generate_move(position: Position) -> Result<Move, ServerFnError> {
    println!("Called!!!!");
    Ok(find_move(&position, 3, MOVE_GEN, POSITION_EVALUATOR))
}

#[component]
pub fn IndexPage() -> impl IntoView {
    let (position, set_position) = create_signal(Position::start());
    let (side, set_side) = create_signal(Side::White);

    let handle_move = create_action(move |input: &Move| {
        web_sys::console::log_1(&format!("Called with {:?}", input).into());
        set_position.update(|pos| pos.make_move(&input).unwrap() );
        async move {
            let generated_move = generate_move(position()).await.unwrap();
            set_position.update(|pos| pos.make_move(&generated_move).unwrap() );
        }
    });

    view! {
        <div class="grid grid-cols-5">
            <div class="bg-gray-200 p-2">
                <h1 class="text-xl font-bold">"vs. computer"</h1>
            </div>
            <div class="col-span-3 flex justify-center">
                <ChessBoard position=position player_side=side handle_move={handle_move} />
            </div>
            <div class="bg-gray-200 p-2">
                <h3 class="text-xl font-bold">"move generation"</h3>
            </div>
        </div>
    }
}
