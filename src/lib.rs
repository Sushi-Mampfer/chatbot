mod app;

mod api;
mod chat;

#[cfg(feature = "ssr")]
mod ssr {
    pub use crate::api::handle_upgrade;
    pub use crate::app::{shell, App};

    pub use std::sync::Arc;

    pub use axum::{routing::get, Extension, Router};
    pub use leptos::prelude::*;
    pub use leptos_axum::{generate_route_list, LeptosRoutes};
    pub use tower_service::Service;
}

#[cfg(feature = "ssr")]
#[worker::event(fetch)]
async fn fetch(
    req: worker::HttpRequest,
    env: worker::Env,
    _ctx: worker::Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
    use crate::ssr::*;

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    // build our application with a route
    let mut router = Router::new()
        .route("/api/chat", get(handle_upgrade))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .with_state(leptos_options)
        .layer(Extension(Arc::new(env))); // <- Allow leptos server functions to access Worker stuff

    Ok(router.call(req).await?)
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    leptos::mount::hydrate_body(app::App);
}
