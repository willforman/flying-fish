use std::collections::HashMap;
use std::fmt::format;
use std::str::FromStr;

use leptos::*;

use engine::bitboard::{Square, SquareIter};
use engine::move_gen::{GenerateMoves, HYPERBOLA_QUINTESSENCE_MOVE_GEN};
use engine::position::{Move, Position, Side};
use leptos::ev::MouseEvent;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

const BG_DARK: &str = "bg-[#86A666]";
const BG_LIGHT: &str = "bg-[#FFFFDD]";

#[component]
pub fn ChessBoard(
    position: ReadSignal<Position>,
    player_side: ReadSignal<Side>,
    #[prop(into)] handle_move: Callback<Move>,
) -> impl IntoView {
    let (selected_square, set_selected_square) = create_signal::<Option<Square>>(None);

    let moves_map = create_memo(move |_| {
        let moves = HYPERBOLA_QUINTESSENCE_MOVE_GEN.gen_moves(&position());
        let mut moves_map = HashMap::new();

        for mve in moves {
            moves_map
                .entry(mve.src)
                .or_insert_with(Vec::new)
                .push(mve.dest);
        }

        moves_map
    });
    let move_dests = move || match selected_square() {
        Some(sq) => moves_map().get(&sq).cloned().unwrap_or(Vec::new()),
        None => Vec::new(),
    };

    let squares = move || {
        if player_side() == Side::White {
            Square::list_white_perspective()
        } else {
            Square::list_black_perspective()
        }
    };

    let handle_square_click = move |mouse_event: MouseEvent| {
        let target = mouse_event
            .target()
            .unwrap_throw()
            .unchecked_into::<web_sys::HtmlElement>();

        let square_str = if target.id() != "" {
            // Empty square was clicked, just return it's id
            target.id()
        } else {
            // Image or move indicator was clicked, get the id of the square it's in
            target.parent_element().unwrap().id()
        };

        web_sys::console::log_1(&format!("{}", square_str).into());

        let square = Square::from_str(&square_str).unwrap_throw();

        if move_dests().contains(&square) {
            let mve = Move::new(selected_square().unwrap(), square);
            handle_move(mve);
            set_selected_square(None);
        } else {
            set_selected_square(Some(square));
        }
    };

    view! {
        <div class="grid grid-cols-8 auto-rows-[1fr] gap-0">
            {move || squares().into_iter()
                .enumerate()
                .map(|(i, square)| {
                    let bg_color = if (i % 2) == (i / 8 % 2) {
                        BG_LIGHT
                    } else {
                        BG_DARK
                    };

                    let class = format!("w-full h-full {} relative p-[0.4rem]", bg_color);
                    let id = square.to_string();

                    if let Some((piece, side)) = position().is_piece_at(square) {
                        let img_name = format!("{}_{}.svg", piece.to_string().to_lowercase(), side.to_string().to_lowercase());
                        let alt = format!("{} {}", piece.to_string(), side.to_string());
                        let class = if move_dests().contains(&square) {
                            format!("{} box-border border-4 border-green-900", class)
                        } else {
                            class
                        };
                        view! {
                            <div class=class on:click={handle_square_click} id={id} >
                                <img src=img_name alt=alt height="78" width="78" />
                            </div>
                        }
                    } else if move_dests().contains(&square) {
                        view! {
                            <div class=class on:click={handle_square_click} id={id} >
                                <div class="w-1/3 h-1/3 rounded-full bg-green-900 absolute top-1/2 left-1/2 opacity-50 transform -translate-x-1/2 -translate-y-1/2" />
                            </div>
                        }
                    } else {
                        view! { <div class=class on:click={handle_square_click} id={id} /> }
                    }

                }).collect_view()
            }
        </div>
    }
}
