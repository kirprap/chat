use actix::Actor;
use actix_web::{test, web, App};
use actix_web::http::StatusCode;
use futures::StreamExt;
use actix_web_actors::ws;
use serde_json::json;

use crate::{websocket_route, AppState, ChatMessage};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[actix_web::test]
async fn test_websocket_connection() {
    // Create app state
    let app_state = web::Data::new(Arc::new(Mutex::new(AppState {
        clients: HashMap::new(),
    })));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/ws/", web::get().to(websocket_route))
    ).await;

    // Create test request
    let req = test::TestRequest::with_uri("/ws/?username=testuser")
        .header("connection", "upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .header("sec-websocket-version", "13")
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::SWITCHING_PROTOCOLS);
}

#[actix_web::test]
async fn test_chat_message_broadcast() {
    // Create app state
    let app_state = web::Data::new(Arc::new(Mutex::new(AppState {
        clients: HashMap::new(),
    })));
    
    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/ws/", web::get().to(websocket_route))
    ).await;

    // Create two test clients
    let mut client1 = test::start_with(
        test::TestRequest::with_uri("/ws/?username=user1")
            .header("connection", "upgrade")
            .header("upgrade", "websocket")
            .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("sec-websocket-version", "13")
            .to_request(),
        |req| {
            let state = app_state.clone();
            websocket_route(req, web::Payload::None, state)
        },
    ).await;

    let mut client2 = test::start_with(
        test::TestRequest::with_uri("/ws/?username=user2")
            .header("connection", "upgrade")
            .header("upgrade", "websocket")
            .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("sec-websocket-version", "13")
            .to_request(),
        |req| {
            let state = app_state.clone();
            websocket_route(req, web::Payload::None, state)
        },
    ).await;

    // Send a message from client1
    let message = ChatMessage {
        username: "user1".to_string(),
        content: "Hello!".to_string(),
        message_type: "message".to_string(),
    };

    client1.send(ws::Message::Text(
        serde_json::to_string(&message).unwrap().into(),
    )).await;

    // Check if client2 receives the message
    if let Some(Ok(ws::Frame::Text(bytes))) = client2.next().await {
        let received: ChatMessage = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(received.username, "user1");
        assert_eq!(received.content, "Hello!");
        assert_eq!(received.message_type, "message");
    } else {
        panic!("Expected text message");
    }
}

#[actix_web::test]
async fn test_user_join_leave_messages() {
    // Create app state
    let app_state = web::Data::new(Arc::new(Mutex::new(AppState {
        clients: HashMap::new(),
    })));
    
    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/ws/", web::get().to(websocket_route))
    ).await;

    // Create a test client that will receive notifications
    let mut client = test::start_with(
        test::TestRequest::with_uri("/ws/?username=observer")
            .header("connection", "upgrade")
            .header("upgrade", "websocket")
            .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("sec-websocket-version", "13")
            .to_request(),
        |req| {
            let state = app_state.clone();
            websocket_route(req, web::Payload::None, state)
        },
    ).await;

    // Create another client to trigger join/leave messages
    let new_client = test::start_with(
        test::TestRequest::with_uri("/ws/?username=newuser")
            .header("connection", "upgrade")
            .header("upgrade", "websocket")
            .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("sec-websocket-version", "13")
            .to_request(),
        |req| {
            let state = app_state.clone();
            websocket_route(req, web::Payload::None, state)
        },
    ).await;

    // Check for join message
    if let Some(Ok(ws::Frame::Text(bytes))) = client.next().await {
        let received: ChatMessage = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(received.username, "newuser");
        assert_eq!(received.message_type, "join");
        assert_eq!(received.content, "joined the chat");
    } else {
        panic!("Expected join message");
    }

    // Drop the new client to trigger a leave message
    drop(new_client);

    // Check for leave message
    if let Some(Ok(ws::Frame::Text(bytes))) = client.next().await {
        let received: ChatMessage = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(received.username, "newuser");
        assert_eq!(received.message_type, "leave");
        assert_eq!(received.content, "left the chat");
    } else {
        panic!("Expected leave message");
    }
}