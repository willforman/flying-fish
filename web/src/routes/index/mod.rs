use leptos::*;

pub mod chess_board;

use crate::routes::index::chess_board::ChessBoard;
use engine::position::Position;

#[component]
pub fn IndexPage() -> impl IntoView {
    let (position, set_position) = create_signal(Position::start());

    view! {
        <div class="grid grid-cols-5">
            <div class="bg-gray-200 p-2">
                <h1 class="text-xl font-bold">"vs. computer"</h1>
            </div>
            <div class="col-span-3 flex justify-center">
                <ChessBoard position=position/>
            </div>
            <div>
                <h3>"move generation"</h3>
            </div>
        </div>
    }
}
