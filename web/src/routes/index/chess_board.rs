use std::str::FromStr;

use leptos::*;

use engine::bitboard::Square;
use engine::position::{Position, Side};
use leptos::ev::MouseEvent;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

const BG_DARK: &str = "bg-[#86A666]";
const BG_LIGHT: &str = "bg-[#FFFFDD]";

#[component]
pub fn ChessBoard(position: ReadSignal<Position>, player_side: ReadSignal<Side>) -> impl IntoView {
    let squares = if player_side() == Side::White {
        Square::list_white_perspective()
    } else {
        Square::list_black_perspective()
    };

    let handle_square_click = move |mouse_event: MouseEvent| {
        let square_str = mouse_event
            .target()
            .unwrap_throw()
            .unchecked_into::<web_sys::HtmlImageElement>()
            .id();

        let square = Square::from_str(&square_str).unwrap_throw();
        panic!("{:?}", square);
    };

    view! {
        <div class="grid grid-cols-8 auto-rows-[1fr] gap-0">
            {squares.into_iter()
                .enumerate()
                .map(|(i, square)| {
                    let bg_color = if (i % 2) == (i / 8 % 2) {
                        BG_LIGHT
                    } else {
                        BG_DARK
                    };
                    let class = format!("w-full h-full {}", bg_color);

                    if let Some((piece, side)) = position().is_piece_at(square) {
                        let img_name = format!("{}_{}.svg", piece.to_string().to_lowercase(), side.to_string().to_lowercase());
                        let alt = format!("{} {}", piece.to_string(), side.to_string());
                        view! {
                            <div class=class>
                                <img src=img_name id={square.to_string()} alt=alt height="78" width="78" on:click={handle_square_click} />
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
