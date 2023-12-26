use leptos::*;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    view! {
        <div class="bg-red-500">
            <h1 class="text-red">Test2</h1>
            {children()}
        </div>
    }
}
