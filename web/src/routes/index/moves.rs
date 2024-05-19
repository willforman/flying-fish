use leptos::*;

#[component]
pub fn Moves(move_strs: ReadSignal<Vec<String>>) -> impl IntoView {
    let move_turns = move || {
        let res: Vec<(String, Option<String>)> = move_strs()
            .chunks(2)
            .map(|chunk| match chunk {
                [a, b] => (a.clone(), Some(b.clone())),
                [a] => (a.clone(), None),
                _ => unreachable!(),
            })
            .collect();
        res
    };

    view! {
        <div class="flex-initial flex flex-col bg-gray-200 p-2 h-full">
            <h1 class="text-xl font-bold mb-4">"vs. computer"</h1>
            <div class="flex flex-col w-full">
                {move || move_turns()
                    .iter()
                    .enumerate()
                    .map(|(idx, (white_mve, maybe_black_mve))| {
                        view! {
                            <div class="flex">
                                <div class="basis-[30%] font-semibold">{format!("{}.",idx)}</div>
                                <div class="basis-[35%]">{white_mve}</div>
                                <div class="basis-[35%]">{maybe_black_mve.clone().unwrap_or("".to_string())}</div>
                            </div>
                        }
                    }).collect_view()
                }
            </div>
        </div>
    }
}
