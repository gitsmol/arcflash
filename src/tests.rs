use log::debug;

/*
pub(crate) fn test_q_all_params() {
    let config_file = PathBuf::from("./config.toml");
    let config = Arc::new(read_config_from_file(&config_file).unwrap());

    let config_ = config.clone();

    let timer = async_std::task::sleep(std::time::Duration::from_secs(3));
    let packet_counter_task = async_std::task::spawn(async move {
        let mut packet_count = 0;
        packet_counter(config.clone(), &mut packet_count).await;
        println!()
    });

    async_std::task::spawn(async move { req_all_params(config_).await });

    loop {}
}

async fn packet_counter(config: Arc<Config>, count: &mut usize) -> async_osc::Result<()> {
    let mut socket = OscSocket::bind(config.instrument.local_addr()).await?;
    // Listen for incoming packets on the socket.
    while let Some(_) = socket.next().await {
        *count += 1;
        debug!("Received {count} messages.");
    }
    Ok(())
}

async fn req_all_params(config: Arc<Config>) -> async_osc::Result<()> {
    println!("Requesting all params");
    let message = OscMessage::new("/q/all_params", OscType::Float(0.0));
    let peer_send = Arc::new(config.instrument.clone());
    sender::send_message(message, peer_send).await;

    Ok(())
}

async fn random_patch() -> async_osc::Result<()> {
    println!("Requesting random patch");
    let socket = OscSocket::bind("192.168.1.110:53102").await?;
    let message = OscMessage::new("/patch/random", OscType::Float(0.0));

    socket.send_to(message, "192.168.1.103:53100").await?;
    Ok(())
}
 */
