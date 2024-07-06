use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use leptos::*;

pub mod chess_board;
pub mod moves;

use engine::{
    move_to_algebraic_notation, search, HyperbolaQuintessenceMoveGen, Move, Position, SearchParams,
    SearchResultInfo, Side, HYPERBOLA_QUINTESSENCE_MOVE_GEN, POSITION_EVALUATOR,
};
use leptos::html;
use leptos::logging::log;
use web_sys::SubmitEvent;

use crate::routes::index::chess_board::ChessBoard;
use crate::routes::index::moves::Moves;

static MOVE_GEN: HyperbolaQuintessenceMoveGen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;

#[server(GenerateMove)]
async fn generate_move(
    position: Position,
    search_params: SearchParams,
) -> Result<(Option<Move>, SearchResultInfo), ServerFnError> {
    let terminate = Arc::new(AtomicBool::new(false));
    let info_holder: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let (best_move, positions_processed) = search(
        &position,
        &search_params,
        MOVE_GEN,
        POSITION_EVALUATOR,
        Arc::clone(&info_holder),
        Arc::clone(&terminate),
    )?;
    Ok((best_move, positions_processed))
}

#[component]
pub fn IndexPage() -> impl IntoView {
    let (game_complete, set_game_complete) = create_signal(false);
    let (position, set_position) = create_signal(Position::start());
    let (side, set_side) = create_signal(Side::White);
    let (move_strs, set_move_strs) = create_signal(Vec::<String>::new());

    let search_params = SearchParams {
        move_time: Some(Duration::from_secs(5)),
        ..SearchParams::default()
    };

    let handle_move = create_action(move |input: &Move| {
        let move_str = move_to_algebraic_notation(&position(), input, MOVE_GEN, MOVE_GEN).unwrap();
        set_move_strs.update(|move_strs| move_strs.push(move_str));

        set_position.update(|pos| pos.make_move(input).unwrap());

        let search_params_clone = search_params.clone();

        async move {
            let (maybe_generated_move, search_info) =
                generate_move(position(), search_params_clone)
                    .await
                    .unwrap();
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
