#![cfg(feature = "ssr")]

use serde::{Deserialize, Serialize};
use worker::{
    durable_object, DurableObject, Env, Request, Response, State, WebSocketIncomingMessage,
    WebSocketPair,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct AiInput {
    pub messages: Vec<Message>,
}

#[derive(Deserialize)]
pub struct AiOutput {
    pub choices: Vec<AiChoice>,
}

#[derive(Deserialize)]
pub struct AiChoice {
    pub message: AiMessage,
}

#[derive(Deserialize)]
pub struct AiMessage {
    pub content: String,
}

#[durable_object]
pub struct Chat {
    state: State,
    env: Env,
}

impl DurableObject for Chat {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&self, _req: Request) -> worker::Result<Response> {
        let WebSocketPair { client, server } = WebSocketPair::new()?;

        self.state.accept_web_socket(&server);

        let mut messages: Vec<Message> = self
            .state
            .storage()
            .get("messages")
            .await?
            .unwrap_or_else(|| {
                vec![Message {
                    role: "system".to_string(),
                    content: "You are a friendly assistant".to_string(),
                }]
            });

        messages.retain(|m| m.role != "system");
        server.send(&messages)?;

        Response::from_websocket(client)
    }

    async fn websocket_close(
        &self,
        _ws: worker::WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> worker::Result<()> {
        Ok(())
    }

    async fn websocket_message(
        &self,
        ws: worker::WebSocket,
        message: worker::WebSocketIncomingMessage,
    ) -> worker::Result<()> {
        let WebSocketIncomingMessage::String(mut msg) = message else {
            return Ok(());
        };
        msg.remove(0);
        msg.remove(msg.len() - 1);

        let ai = self.env.ai("AI")?;

        let mut messages: Vec<Message> = self
            .state
            .storage()
            .get("messages")
            .await?
            .unwrap_or_else(|| {
                vec![Message {
                    role: "system".to_string(),
                    content: "You are a friendly assistant".to_string(),
                }]
            });

        messages.push(Message {
            role: "user".to_string(),
            content: msg,
        });

        let out: AiOutput = ai
            .run(
                "@cf/ibm-granite/granite-4.0-h-micro",
                AiInput {
                    messages: messages.clone(),
                },
            )
            .await?;

        messages.push(Message {
            role: "assistant".to_string(),
            content: out.choices[0].message.content.clone(),
        });

        self.state
            .storage()
            .put("messages", messages.clone())
            .await?;

        messages.retain(|m| m.role != "system");
        ws.send(&messages)?;

        Ok(())
    }
}
