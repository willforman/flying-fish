use std::collections::HashMap;
use std::str::FromStr;

use leptos::*;

use engine::bitboard::Square;
use engine::move_gen::{GenerateMoves, HYPERBOLA_QUINTESSENCE_MOVE_GEN};
use engine::position::{Move, Position, Side};
use leptos::ev::MouseEvent;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

const BG_DARK: &str = "bg-[#86A666]";
const BG_LIGHT: &str = "bg-[#FFFFDD]";

#[component]
pub fn ChessBoard(position: ReadSignal<Position>, player_side: ReadSignal<Side>) -> impl IntoView {
    let (move_dests, set_move_dests) = create_signal(Vec::<Square>::new());
    let moves_map = create_memo(move |_| {
        let move_set = HYPERBOLA_QUINTESSENCE_MOVE_GEN.gen_moves(&position());
        let mut moves_map = HashMap::new();

        for mve in move_set {
            moves_map
                .entry(mve.src)
                .or_insert_with(Vec::new)
                .push(mve.dest);
        }

        moves_map
    });

    // let squares = if (move || player_side())() == Side::White {
    //     Square::list_white_perspective()
    // } else {
    //     Square::list_black_perspective()
    // };
    let squares = Square::list_white_perspective();

    let handle_square_click = move |mouse_event: MouseEvent| {
        let square_str = mouse_event
            .target()
            .unwrap_throw()
            .unchecked_into::<web_sys::HtmlImageElement>()
            .id();

        let square = Square::from_str(&square_str).unwrap_throw();

        if move_dests().contains(&square) {
            todo!();
        } else {
            let got_move_dests = moves_map().get(&square).cloned().unwrap_or(Vec::new());
            set_move_dests.set(got_move_dests);
        }
    };

    view! {
        <div class="grid grid-cols-8 auto-rows-[1fr] gap-0">
            {move || squares.into_iter()
                .enumerate()
                .map(|(i, square)| {
                    let bg_color = if (i % 2) == (i / 8 % 2) {
                        BG_LIGHT
                    } else {
                        BG_DARK
                    };
                    let class = format!("w-full h-full {} relative", bg_color);
                    let is_piece_at = position().is_piece_at(square);

                    if let Some((piece, side)) = is_piece_at {
                        let img_name = format!("{}_{}.svg", piece.to_string().to_lowercase(), side.to_string().to_lowercase());
                        let alt = format!("{} {}", piece.to_string(), side.to_string());
                        view! {
                            <div class=class>
                                <img src=img_name id={square.to_string()} alt=alt height="78" width="78" on:click={handle_square_click} />
                            </div>
                        }
                    } else if (move || move_dests().contains(&square))() {
                        view! {
                            <div class=class >
                                <div class="w-2/5 h-2/5 rounded-full bg-green-900 absolute top-1/2 left-1/2 opacity-50 transform -translate-x-1/2 -translate-y-1/2" />
                            </div>
                        }
                    } else {
                        view! { <div class=class /> }
                    }

                }).collect_view()
            }
        </div>
    }
}
