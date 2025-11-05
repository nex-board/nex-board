use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade}, 
    response::IntoResponse, 
    routing::get, 
    Extension, 
    Router
};
use bevy_tokio_tasks::TokioTasksRuntime;
use bevy::prelude::*;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum WsCommand {
    Bulletin {
	preset: String,
	index: u32,
    },
    Bingo {
	method: BingoMethod,
    },
    Countdown {
	method: CountdownMethod,
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum BingoMethod {
    Next,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CountdownMethod {
    Start,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum WsResponse {
    Bulletin(BulletinResponse),
    Bingo(BingoResponse),
    Countdown(CountdownResponse),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BulletinResponse {
    #[serde(rename = "prev-text")]
    pub prev_text: String,
    #[serde(rename = "now-text")]
    pub now_text: String,
    #[serde(rename = "next-text")]
    pub next_text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BingoResponse {
    pub current: u8,
    pub no: u8,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CountdownResponse {
    pub status: String,
}

#[derive(Resource)]
pub struct WebSocketChannel {
    pub command_receiver: mpsc::Receiver<WsCommand>,
    pub response_sender: mpsc::Sender<WsResponse>,
}

#[derive(Resource)]
pub struct CommandSender {
    pub sender: mpsc::Sender<WsCommand>,
}

pub fn setup_websocket_server(app: &mut App) {
    let (command_tx, command_rx) = mpsc::channel::<WsCommand>(100);
    let (response_tx, _response_rx) = mpsc::channel::<WsResponse>(100);
    
    app.insert_resource(CommandSender {
        sender: command_tx,
    });
    
    app.insert_resource(WebSocketChannel {
        command_receiver: command_rx,
        response_sender: response_tx,
    });
    
    app.add_systems(Startup, start_axum_server);
    app.add_systems(Update, handle_websocket_commands);
}

fn start_axum_server(
    runtime: Res<TokioTasksRuntime>,
    command_sender: Res<CommandSender>,
) {
    let command_tx = command_sender.sender.clone();
    
    runtime.spawn_background_task(move |_ctx| async move {
        let app = Router::new()
            .route("/ws", get(ws_handler))
            .layer(Extension(command_tx));
            
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
            .await
            .expect("Failed to bind to address");
            
        println!("WebSocket server running on ws://0.0.0.0:3000/ws");
        
        axum::serve(listener, app)
            .await
            .expect("Server failed to start");
    });
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(command_tx): Extension<mpsc::Sender<WsCommand>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, command_tx))
}

async fn handle_websocket(
    mut socket: WebSocket,
    command_tx: mpsc::Sender<WsCommand>,
) {
    // クライアントからのメッセージを受信
    while let Some(result) = socket.recv().await {
        match result {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<WsCommand>(&text) {
                    Ok(command) => {
                        if command_tx.send(command).await.is_err() {
                            eprintln!("Failed to send command to Bevy");
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse WebSocket message: {}", e);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("WebSocket connection closed");
                break;
            }
            Ok(_) => {} // その他のメッセージタイプは無視
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }
}

fn handle_websocket_commands(
    mut commands: Commands,
    mut ws_channel: ResMut<WebSocketChannel>,
    mut text_queue: ResMut<crate::TextQueue>,
    mut bingo_state: ResMut<crate::bingo::BingoState>,
    mut scrolling_state: ResMut<crate::ScrollingState>,
    mut scrolling_speed: ResMut<crate::ScrollingSpeed>,
    config: Res<crate::loader::Config>,
    fonts: Res<crate::Fonts>,
    text_query: Query<Entity, With<crate::Showing>>,
) {
    while let Ok(command) = ws_channel.command_receiver.try_recv() {
        match command {
            WsCommand::Bulletin { preset: _, index } => {
                // 現在のテキストを削除
                for entity in text_query.iter() {
                    commands.entity(entity).despawn();
                }
                
                // 新しいテキストをスポーン
                if let Some(text_source) = text_queue.texts.get(index as usize) {
                    let text_content = text_source.content.clone();
                    let text_duration = text_source.duration;
                    
                    crate::text_spawner::spawn_text(
                        &mut commands,
                        &text_content,
                        &text_duration,
                        fonts.text_font.clone(),
                        &config,
                        &mut *scrolling_speed,
                    );
                    
                    text_queue.current_index = index as usize;
                    scrolling_state.is_active = true;
                    
                    // レスポンスを送信
                    let prev_text = text_queue.texts.get(index.saturating_sub(1) as usize)
                        .map(|t| t.content.clone())
                        .unwrap_or_default();
                    let now_text = text_content;
                    let next_text = text_queue.texts.get((index + 1) as usize)
                        .map(|t| t.content.clone())
                        .unwrap_or_default();
                    
                    let response = WsResponse::Bulletin(BulletinResponse {
                        prev_text,
                        now_text,
                        next_text,
                    });
                    
                    let _ = ws_channel.response_sender.try_send(response);
                }
            }
            WsCommand::Bingo { method } => {
                match method {
                    BingoMethod::Next => {
                        // 現在のテキストを削除
                        for entity in text_query.iter() {
                            commands.entity(entity).despawn();
                        }
                        
                        if let Some(number) = bingo_state.next() {
                            crate::text_spawner::spawn_static_text(
                                &mut commands,
                                &number.to_string(),
                                fonts.text_font.clone(),
                            );
                            
                            let response = WsResponse::Bingo(BingoResponse {
                                current: number,
                                no: bingo_state.index as u8,
                            });
                            
                            let _ = ws_channel.response_sender.try_send(response);
                        }
                    }
                }
            }
            WsCommand::Countdown { method } => {
                match method {
                    CountdownMethod::Start => {
                        // カウントダウン開始のロジックをここに実装
                        // 現在のプロジェクトにはcountdownモジュールがあるようなので、
                        // それを使用することを想定
                        
                        let response = WsResponse::Countdown(CountdownResponse {
                            status: "started".to_string(),
                        });
                        
                        let _ = ws_channel.response_sender.try_send(response);
                    }
                }
            }
        }
    }
}
    
