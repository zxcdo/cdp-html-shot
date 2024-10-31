use tokio::time;
use time::Duration;
use serde_json::Value;
use futures_util::StreamExt;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::connect_async;

use crate::transport_actor::{TransportActor, TransportMessage, TransportResponse};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Response {
    pub(crate) id: u64,
    pub(crate) result: Value,
}

#[derive(Debug)]
pub(crate) struct Transport {
    tx: mpsc::Sender<TransportMessage>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Transport {
    pub(crate) async fn new(ws_url: &str) -> Result<Self> {
        let (ws_stream, _) = connect_async(ws_url).await?;
        let (ws_sink, ws_stream) = ws_stream.split();

        let (tx, rx) = mpsc::channel::<TransportMessage>(100);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let actor = TransportActor {
            pending_requests: HashMap::new(),
            ws_sink,
            command_rx: rx,
            shutdown_rx,
        };

        tokio::spawn(actor.run(ws_stream));

        Ok(Self { tx, shutdown_tx: Some(shutdown_tx) })
    }

    pub(crate) async fn send(&self, command: Value) -> Result<TransportResponse> {
        let (response_tx, response_rx) = oneshot::channel();

        self.tx.send(TransportMessage::Request(command, response_tx)).await?;

        match time::timeout(Duration::from_secs(5), response_rx).await {
            Ok(response) => response?,
            Err(_) => Err(anyhow!("Timeout while waiting for response")),
        }
    }

    pub(crate) async fn get_target_msg(&self, msg_id: usize) -> Result<TransportResponse> {
        let (response_tx, response_rx) = oneshot::channel();

        self.tx.send(TransportMessage::ListenTargetMessage(msg_id as u64, response_tx)).await?;

        match time::timeout(Duration::from_secs(5), response_rx).await {
            Ok(response) => response?,
            Err(_) => Err(anyhow!("Timeout while waiting for response")),
        }
    }

    pub(crate) fn shutdown(&mut self) {
        self.shutdown_tx
            .take()
            .unwrap()
            .send(())
            .unwrap();
    }
}