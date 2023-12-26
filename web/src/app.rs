use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::layout::Layout;
use crate::routes::index::IndexPage;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/web.css"/>
        <Title text="Chess"/>

        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main>
                <h1 class="text-red text-xl">SHOULD BE RED</h1>
                <Layout> 
                    <Routes>
                        <Route path="" view=IndexPage/>
                    </Routes>
                </Layout>
            </main>
        </Router>
    }
}
