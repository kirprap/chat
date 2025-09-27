#[cfg(test)]
mod integration_tests {
    use actix::Actor;
    use actix_web::{web, App, HttpServer};
    use futures::{SinkExt, StreamExt};
    use gloo_net::websocket::futures::WebSocket;
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::time::sleep;
    use crate::{AppState, ChatMessage};
    use std::collections::HashMap;

    #[actix_web::test]
    async fn test_full_chat_flow() {
        // Start the server
        let app_state = Arc::new(Mutex::new(AppState {
            clients: HashMap::new(),
        }));
        
        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .route("/ws/", web::get().to(crate::websocket_route))
        })
        .bind(("127.0.0.1", 8081))
        .unwrap()
        .run();

        // Give the server time to start
        sleep(Duration::from_millis(100)).await;

        // Connect two WebSocket clients
        let ws1 = WebSocket::open("ws://127.0.0.1:8081/ws/?username=user1").unwrap();
        let ws2 = WebSocket::open("ws://127.0.0.1:8081/ws/?username=user2").unwrap();

        let (mut write1, mut read1) = ws1.split();
        let (mut write2, mut read2) = ws2.split();

        // User1 sends a message
        let message = ChatMessage {
            username: "user1".to_string(),
            content: "Hello from integration test!".to_string(),
            message_type: "message".to_string(),
        };

        write1.send(gloo_net::websocket::Message::Text(
            serde_json::to_string(&message).unwrap()
        )).await.unwrap();

        // User2 should receive the message
        if let Some(Ok(gloo_net::websocket::Message::Text(msg))) = read2.next().await {
            let received: ChatMessage = serde_json::from_str(&msg).unwrap();
            assert_eq!(received.username, "user1");
            assert_eq!(received.content, "Hello from integration test!");
            assert_eq!(received.message_type, "message");
        } else {
            panic!("Expected message not received");
        }

        // Cleanup
        drop(write1);
        drop(write2);
        drop(read1);
        drop(read2);
    }
}