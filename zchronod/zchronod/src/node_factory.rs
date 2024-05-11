use std::sync::{Arc};
use node_api::config::ZchronodConfig;
use node_api::error::ZchronodResult;
use tokio::net::UdpSocket;
use tokio::sync::{Mutex, RwLock};
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::sleep;
use crate::{storage, zchronod};
use crate::zchronod::{ServerState, Zchronod, ZchronodArc};

#[derive(Default)]
pub struct ZchronodFactory {
    pub config: ZchronodConfig,
}

impl ZchronodFactory {
    pub fn init() -> Self {
        Self::default()
    }

    pub fn set_config(mut self, config: ZchronodConfig) -> Self {
        self.config = config;
        self
    }

    pub async fn produce(self) -> ZchronodResult<ZchronodArc> {
        let storage = storage::Storage::new(self.config.clone()).await;

        /// p2p network pass send && recv
        // let (p2p_send, p2p_recv) = match zchronod_p2p::spawn_zchronod_p2p().await;

        let cfg = self.config.clone();
        let socket = UdpSocket::bind(self.config.inner_p2p).await.unwrap();
        let state = ServerState::new("".to_owned());
        // create arc zchronod node
        let arc_zchronod = Arc::new(RwLock::new(Zchronod {
            config: cfg,
            socket,
            storage,
            state
        }));
        
        let mut join_handles: Vec<JoinHandle<()>> = Vec::new();
        join_handles.push(tokio::spawn(zchronod::p2p_event_loop(arc_zchronod.clone())));
        
        // start client websocket
        join_handles.push(tokio::spawn(zchronod::handle_incoming_ws_msg(self.config.ws_url)));

        for handle in join_handles {
            handle.await.unwrap();
        }

        ZchronodResult::Ok(arc_zchronod)
    }
}