use leptos::*;

pub mod chess_board;

use crate::routes::index::chess_board::ChessBoard;

#[component]
pub fn IndexPage() -> impl IntoView {
    view! {
        <h1 class="text-center">"Vs. Computer"</h1>
        <ChessBoard />
    }
}
