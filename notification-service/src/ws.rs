use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{app_state::AppState, hub::WsHub};

#[derive(Deserialize)]
pub struct WsQuery {
    pub order_id: Option<i64>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let order_id = query.order_id.unwrap_or(0);
    ws.on_upgrade(move |socket| handle_socket(socket, state.hub, order_id))
}

async fn handle_socket(socket: WebSocket, hub: WsHub, order_id: i64) {
    let conn_id = Uuid::new_v4().to_string();
    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    hub.add_client(order_id, conn_id.clone(), tx).await;
    tracing::info!("WebSocket connected: order_id={}, conn_id={}", order_id, conn_id);

    let send_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender.send(Message::Text(message.into())).await.is_err() {
                break;
            }
        }
    });

    let hub_for_recv = hub.clone();
    let conn_id_for_recv = conn_id.clone();

    let recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Close(_)) => break,
                Ok(_) => {} 
                Err(err) => {
                    tracing::warn!("ws receive error for {conn_id_for_recv}: {err}");
                    break;
                }
            }
        }

        hub_for_recv.remove_client(order_id, &conn_id_for_recv).await;
    });

    let _ = tokio::join!(send_task, recv_task);
    hub.remove_client(order_id, &conn_id).await;
    tracing::info!("WebSocket disconnected: order_id={}, conn_id={}", order_id, conn_id);
}