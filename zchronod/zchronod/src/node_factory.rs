use std::sync::{Arc, RwLock};
use node_api::config::ZchronodConfig;
use node_api::error::ZchronodResult;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
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

    pub async fn create_zchronod(config: ZchronodConfig) -> ZchronodArc {
        let cfg = config.clone();
        let address = config.inner_p2p.clone();
        let socket = UdpSocket::bind(address).await.unwrap();
        let state = ServerState::new("".to_owned());
        let storage = storage::Storage::new(config.clone()).await;
        let zchronod = Zchronod {
            config: cfg,
            socket,
            storage,
            state,
        };

        Arc::new(Mutex::new(zchronod))
    }

    pub async fn initialize_node(self) -> ZchronodResult<ZchronodArc> {
        let arc_zchronod = ZchronodFactory::create_zchronod(self.config.clone()).await;

        let mut set = JoinSet::new();
        set.spawn(zchronod::p2p_event_loop(arc_zchronod.clone()));
        set.spawn(zchronod::handle_incoming_ws_msg(self.config.ws_url));
        set.join_next().await;
        set.abort_all();

        Ok(arc_zchronod)
    }
}