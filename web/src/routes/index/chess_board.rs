use leptos::*;

use engine::position::Position;

const BG_DARK: &str = "bg-[#86A666]";
const BG_LIGHT: &str = "bg-[#FFFFDD]";

#[component]
pub fn ChessBoard(position: ReadSignal<Position>) -> impl IntoView {
    view! {
        <div class="grid grid-cols-8 gap-0 w-[40rem] h-[40rem]">
            {(0..64).into_iter()
                .map(|i| {
                    let color = if (i % 2) == (i / 8 % 2) {
                        BG_LIGHT
                    } else {
                        BG_DARK
                    };
                    view! { <div class=format!("w-full h-full {}", color) /> }
                }).collect_view()
            }
        </div>
    }
}
