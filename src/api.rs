#![cfg(feature = "ssr")]

use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::Query,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse as _, Response},
    Extension,
};
use worker::{Env, Method, Request};

#[worker::send]
pub async fn handle_upgrade(
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    env: Extension<Arc<Env>>,
) -> Response {
    if headers.get("Upgrade") != Some(&HeaderValue::from_static("websocket")) {
        return StatusCode::from_u16(426).unwrap().into_response();
    }
    let Some(id) = params.get("id") else {
        return StatusCode::from_u16(426).unwrap().into_response();
    };
    let durable = env.durable_object("CHAT").unwrap().get_by_name(id).unwrap();

    let mut req = Request::new("https://fake-host", Method::Get).unwrap();
    req.headers_mut()
        .unwrap()
        .append("Upgrade", "websocket")
        .unwrap();
    let res = durable.fetch_with_request(req).await.unwrap();
    Response::try_from(res).unwrap()
}
