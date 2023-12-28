use leptos::*;

use engine::{bitboard::Square, position::Position};

const BG_DARK: &str = "bg-[#86A666]";
const BG_LIGHT: &str = "bg-[#FFFFDD]";

#[component]
pub fn ChessBoard(position: ReadSignal<Position>) -> impl IntoView {
    view! {
        <div class="grid grid-cols-8 auto-rows-[1fr] gap-0">
            {(0..64).into_iter()
                .map(|i| {
                    let bg_color = if (i % 2) == (i / 8 % 2) {
                        BG_LIGHT
                    } else {
                        BG_DARK
                    };
                    let class = format!("w-full h-full {}", bg_color);

                    let square = Square::from_repr(i).unwrap();
                    if let Some((piece, side)) = position().is_piece_at(square) {
                        let img_name = format!("{}_{}.svg", piece.to_string().to_lowercase(), side.to_string().to_lowercase());
                        let alt = format!("{} {}", piece.to_string(), side.to_string());
                        view! {
                            <div class=class >
                                <img src=img_name alt=alt height="75" width="75" />
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
