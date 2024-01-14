use leptos::*;

use engine::position::Move;

#[component]
pub fn Moves(moves: ReadSignal<Vec<Move>>) -> impl IntoView {
    view! {
        <div class="bg-gray-200 p-2 overflow-scroll">
            <h1 class="text-xl font-bold">"vs. computer"</h1>
            {move || moves().iter()
                .map(|mve| {
                    view! {
                        <div>
                            <p>{format!("{:?}", mve)}</p>
                        </div>
                    }
                }).collect_view()
            }
        </div>
    }
}
