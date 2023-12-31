use std::{path::PathBuf, sync::Arc};

use async_osc::{OscSocket, Result};
use async_std::stream::StreamExt;
use log::debug;

use crate::{
    config::{read_config_from_file, Config},
    peer::PeerKind,
};

pub(crate) fn test_q_all_params() {
    let config_file = PathBuf::from("./config.toml");
    let config = read_config_from_file(&config_file).unwrap();

    async_std::task::block_on(async move {
        let mut packet_count = 0;
        packet_counter(config, &mut packet_count).await.unwrap();
    });
}

async fn packet_counter(config: Config, count: &mut usize) -> async_osc::Result<()> {
    let mut socket = OscSocket::bind(config.instrument.listen_addr()).await?;
    // Listen for incoming packets on the socket.
    while let Some(packet) = socket.next().await {
        let (message, _) = packet?;
        *count += 1;
        debug!("Received {count} messages.");
    }
    Ok(())
}
