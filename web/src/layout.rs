use leptos::*;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    view! {
        <div class="bg-gray-100 h-screen p-8">
            {children()}
        </div>
    }
}
