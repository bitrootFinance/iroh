use std::net::SocketAddr;

use anyhow::{bail, Error};

#[derive(Debug, Clone)]
pub struct PortMapper {}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub pcp: bool,
    pub pmp: bool,
    pub upnp: bool,
}

/// A port mapping client.
#[derive(Debug, Clone)]
pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Client {}
    }

    pub async fn probe(&self) -> Result<ProbeResult, Error> {
        bail!("not implemented yet")
    }

    /// Updates the local port number to which we want to port map UDP traffic.
    pub async fn set_local_port(&self, local_port: u16) {
        // TODO:
    }

    /// Quickly returns with our current cached portmapping, if any.
    /// If there's not one, it starts up a background goroutine to create one.
    /// If the background goroutine ends up creating one, the `on_change` hook registered with the
    /// `Client::new` constructor (if any) will fire.
    pub async fn get_cached_mapping_or_start_creating_one(&self) -> Option<SocketAddr> {
        // TODO:
        None
    }

    pub fn have_mapping(&self) -> bool {
        // TODO:
        false
    }

    pub fn note_network_down(&self) {
        // TODO:
    }

    pub fn close(&self) {
        // TODO:
    }
}
