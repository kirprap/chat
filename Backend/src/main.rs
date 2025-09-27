use actix::prelude::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Message types for chat communication
#[derive(Serialize, Deserialize)]
struct ChatMessage {
    username: String,
    content: String,
    message_type: String, // "message", "join", "leave"
}

// Server state to track connected clients
struct AppState {
    clients: HashMap<String, Recipient<WsMessage>>,
}

// Actor message for internal communication
#[derive(Message)]
#[rtype(result = "()")]
struct WsMessage(String);

// WebSocket actor
struct ChatWs {
    id: String,
    username: String,
    state: Arc<Mutex<AppState>>,
}

impl Actor for ChatWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Store client's recipient
        let recipient = ctx.address().recipient();
        self.state.lock().unwrap().clients.insert(self.id.clone(), recipient);

        // Broadcast join message
        let join_msg = ChatMessage {
            username: self.username.clone(),
            content: "joined the chat".to_string(),
            message_type: "join".to_string(),
        };
        self.broadcast_message(&serde_json::to_string(&join_msg).unwrap());
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // Remove client and broadcast leave message
        self.state.lock().unwrap().clients.remove(&self.id);
        let leave_msg = ChatMessage {
            username: self.username.clone(),
            content: "left the chat".to_string(),
            message_type: "leave".to_string(),
        };
        self.broadcast_message(&serde_json::to_string(&leave_msg).unwrap());
        Running::Stop
    }
}

impl ChatWs {
    fn broadcast_message(&self, msg: &str) {
        if let Ok(state) = self.state.lock() {
            for client in state.clients.values() {
                let _ = client.do_send(WsMessage(msg.to_owned()));
            }
        }
    }
}

// Handle messages received by the actor
impl Handler<WsMessage> for ChatWs {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

// Handle WebSocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                // Parse incoming message
                // Validate message format and broadcast if valid
                if serde_json::from_str::<ChatMessage>(&text).is_ok() {
                    // Broadcast the message to all clients
                    self.broadcast_message(&text);
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

// WebSocket connection handler
async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<Arc<Mutex<AppState>>>,
) -> Result<HttpResponse, Error> {
    // Extract username from query parameters
    let username = req.query_string()
        .split('&')
        .find(|s| s.starts_with("username="))
        .and_then(|s| s.split('=').nth(1))
        .unwrap_or("anonymous")
        .to_string();

    let id = uuid::Uuid::new_v4().to_string();
    let ws = ChatWs {
        id,
        username,
        state: state.get_ref().clone(),
    };

    ws::start(ws, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting chat server at ws://127.0.0.1:8080/ws/");
    
    // Initialize application state
    let app_state = Arc::new(Mutex::new(AppState {
        clients: HashMap::new(),
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/ws/", web::get().to(websocket_route))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}