use crate::{AppState, NotificationChannel};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::interval;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state.notification_tx))
}

async fn handle_socket(socket: WebSocket, tx: NotificationChannel) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast channel
    let mut rx = tx.subscribe();

    // Spawn task to send notifications
    let mut send_task = tokio::spawn(async move {
        // Send periodic pings
        let mut ping_interval = interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                // Receive broadcast notifications
                msg = rx.recv() => {
                    match msg {
                        Ok(notification) => {
                            if sender.send(Message::Text(notification)).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                // Send ping
                _ = ping_interval.tick() => {
                    if sender.send(Message::Ping(vec![])).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Spawn task to receive messages (mainly pongs)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Pong(_) => {
                    // Client is alive
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    tracing::info!("WebSocket connection closed");
}
