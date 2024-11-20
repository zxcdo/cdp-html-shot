use tokio::time;
use time::Duration;
use serde_json::Value;
use futures_util::StreamExt;
use anyhow::{anyhow, Result};
use tokio::sync::{mpsc, oneshot};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::connect_async;
use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
};

use crate::transport_actor::{TransportActor, TransportMessage, TransportResponse};

#[derive(Debug)]
pub(crate) struct ShutdownSignal {
    shutdown: Mutex<bool>,
    condvar: Condvar,
}

impl ShutdownSignal {
    fn new() -> Self {
        ShutdownSignal {
            shutdown: Mutex::new(false),
            condvar: Condvar::new(),
        }
    }

    fn wait(&self) {
        let mut shutdown = self.shutdown.lock().unwrap();
        while !*shutdown {
            shutdown = self.condvar.wait(shutdown).unwrap();
        }
    }

    pub(crate) fn signal_shutdown(&self) {
        let mut shutdown = self.shutdown.lock().unwrap();
        *shutdown = true;
        self.condvar.notify_all();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Response {
    pub(crate) id: u64,
    pub(crate) result: Value,
}

#[derive(Debug)]
pub(crate) struct Transport {
    tx: mpsc::Sender<TransportMessage>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    shutdown_signal: Arc<ShutdownSignal>,
}

unsafe impl Send for Transport {}
unsafe impl Sync for Transport {}

impl Transport {
    pub(crate) async fn new(ws_url: &str) -> Result<Self> {
        let (ws_stream, _) = connect_async(ws_url).await?;
        let (ws_sink, ws_stream) = ws_stream.split();

        let (tx, rx) = mpsc::channel::<TransportMessage>(100);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let signal = Arc::new(ShutdownSignal::new());
        let signal_clone = signal.clone();

        let actor = TransportActor {
            pending_requests: HashMap::new(),
            ws_sink,
            command_rx: rx,
            shutdown_rx,
            shutdown_signal: signal_clone,
        };

        tokio::spawn(actor.run(ws_stream));

        Ok(Self { tx, shutdown_tx: Some(shutdown_tx), shutdown_signal: signal })
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

        self.shutdown_signal.wait();
    }
}