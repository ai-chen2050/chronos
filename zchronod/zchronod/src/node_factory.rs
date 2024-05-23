use std::sync::Arc;
use node_api::config::ZchronodConfig;
use node_api::error::ZchronodResult;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use crate::{handler, storage};
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

    pub async fn create_zchronod(config: ZchronodConfig) -> ZchronodArc {
        let cfg = Arc::new(config.clone());
        let address = config.inner_p2p.clone();
        let node_id = config.node_id.clone().unwrap_or(String::new());
        let socket = UdpSocket::bind(address).await.unwrap();
        let state = RwLock::new(ServerState::new(node_id));
        let storage = storage::Storage::new(cfg.clone()).await;
        let zchronod = Zchronod {
            config: cfg,
            socket,
            storage,
            state,
        };

        Arc::new(zchronod)
    }

    pub async fn initialize_node(self) -> ZchronodResult<ZchronodArc> {
        let arc_zchronod = ZchronodFactory::create_zchronod(self.config.clone()).await;

        let mut join_handles: Vec<JoinHandle<()>> = Vec::new();
        join_handles.push(tokio::spawn(handler::p2p_event_loop(arc_zchronod.clone())));
        
        // start client websocket
        join_handles.push(tokio::spawn(handler::handle_incoming_ws_msg(self.config.ws_url)));

        for handle in join_handles {
            handle.await.unwrap();
        }

        Ok(arc_zchronod)
    }
}