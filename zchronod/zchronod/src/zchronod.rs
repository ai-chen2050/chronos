use crate::vlc::ClockInfo;
use crate::{node_factory::ZchronodFactory, storage::Storage, vlc::Clock};
use node_api::config::ZchronodConfig;
use serde::{Deserialize, Serialize};
use std::{cmp, collections::BTreeSet, sync::Arc};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::*;

pub struct Zchronod {
    pub config: Arc<ZchronodConfig>,
    pub socket: UdpSocket,
    pub storage: Storage,
    pub state: RwLock<ServerState>,
}

pub type ZchronodArc = Arc<Zchronod>;

impl Zchronod {
    pub fn zchronod_factory() -> ZchronodFactory {
        ZchronodFactory::init()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerState {
    pub clock_info: ClockInfo,
    pub node_id: String,
    pub items: BTreeSet<String>,
}

impl ServerState {
    /// Create a new server state.
    pub fn new(node_id: String) -> Self {
        Self {
            clock_info: ClockInfo::new(
                Clock::new(),
                String::new(),
                node_id.clone(),
                "".to_owned(),
                0,
            ),
            node_id,
            items: BTreeSet::new(),
        }
    }

    /// Add items into the state. Returns true if resulting in a new state.
    pub fn add(&mut self, items: BTreeSet<String>) -> bool {
        if items.is_subset(&self.items) {
            info!("duplicate message, no action");
            false
        } else {
            self.items.extend(items);
            self.clock_info.clock.inc(self.node_id.clone());
            self.clock_info.count += 1;
            true
        }
    }

    /// Merge another ServerState into the current state. Returns true if
    /// resulting in a new state (different from current and received
    /// state).
    pub fn merge(&mut self, other: &Self) -> (bool, bool) {
        match self.clock_info.clock.partial_cmp(&other.clock_info.clock) {
            Some(cmp::Ordering::Equal) => (false, false),
            Some(cmp::Ordering::Greater) => (false, false),
            Some(cmp::Ordering::Less) => {
                self.clock_info.clock = other.clock_info.clock.clone();
                self.items = other.items.clone();
                (false, true)
            }
            None => {
                self.clock_info.clock.merge(&vec![&other.clock_info.clock]);
                let added = self.add(other.items.clone());
                (added, added)
            }
        }
    }
}