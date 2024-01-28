use leptos::*;

use engine::position::Move;

#[component]
pub fn Moves(moves: ReadSignal<Vec<Move>>) -> impl IntoView {
    view! {
        <div class="bg-gray-200 p-2">
            <h1 class="text-xl font-bold">"vs. computer"</h1>
            <ul class="max-h-full overflow-scroll">
                {move || (0..30).into_iter()
                    .map(|i| view! { <li><p class="text-xl">{i}</p></li>})
                    .collect_view()
                }
                {move || moves().iter()
                    .map(|mve| {
                        view! {
                            <li>
                                <p>{format!("{:?}", mve)}</p>
                            </li>
                        }
                    }).collect_view()
                }
            </ul>
        </div>
    }
}
