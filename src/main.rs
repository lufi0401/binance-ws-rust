use futures_util::{SinkExt, StreamExt};
use log::*;
use tokio;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error, Message, Result},
};
use url::Url;

const AGENT: &str = "Tungstenite";

#[allow(dead_code)]
async fn get_case_count() -> Result<u32> {
    let (mut socket, _) = connect_async(
        Url::parse("ws://localhost:9001/getCaseCount").expect("Can't connect to case count URL"),
    )
    .await?;
    let msg = socket.next().await.expect("Can't fetch case count")?;
    socket.close(None).await?;
    Ok(msg
        .into_text()?
        .parse::<u32>()
        .expect("Can't parse case count"))
}

#[allow(dead_code)]
async fn update_reports() -> Result<()> {
    let (mut socket, _) = connect_async(
        Url::parse(&format!(
            "ws://localhost:9001/updateReports?agent={}",
            AGENT
        ))
        .expect("Can't update reports"),
    )
    .await?;
    socket.close(None).await?;
    Ok(())
}

#[allow(dead_code)]
async fn run_test(case: u32) -> Result<()> {
    info!("Running test case {}", case);
    let case_url = Url::parse(&format!(
        "ws://localhost:9001/runCase?case={}&agent={}",
        case, AGENT
    ))
    .expect("Bad testcase URL");

    let (mut ws_stream, _) = connect_async(case_url).await?;
    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if msg.is_text() || msg.is_binary() {
            ws_stream.send(msg).await?;
        }
    }

    Ok(())
}

// #[warn(dead_code)]
async fn run_bnce() -> Result<()> {
    info!("Running bnce");
    let bnce_base_url = "wss://stream.binance.com:9443/ws";
    #[allow(unused_variables)]
    let port = 9443;
    let ticker_stream = "bnbbtc@depth";

    let mut connect_url =
        Url::parse(&format!("{}/{}", bnce_base_url, ticker_stream)).expect("Bad Url");
    // connect_url.set_port(Some(port)).expect("cannot set port");
    println!("{}", connect_url.path());
    info!("url,{},port,{:?}", connect_url.as_str(), connect_url.port());

    let (mut ws_stream, _) = connect_async(connect_url).await?;
    info!("Connected");

    let (mut sender, mut recv) = ws_stream.split();

    info!("Split");

    let sub = "{\"method\":\"SUBSCRIBE\",\"params\":[\"BTCBUSD@bookTicker\"],\"id\":1}";
    let result = sender.send(Message::Text(sub.to_string())).await;
    match result {
        Err(err) => info!("send error: {}", err),
        _ => (),
    }

    let mut count = 0;
    while let next = recv.next().await {
        match next {
            Some(Ok(msg)) => {
                if msg.is_text() {
                    info!("recv:{}", msg)
                } else if msg.is_ping() {
                    info!("pong received");
                    _ = sender.send(Message::Pong(Vec::new()));
                } else if msg.is_close() {
                    info!("close received");
                    break;
                } else {
                    info!("len:{}, close:{}", msg.len(), msg.is_close())
                }
            }
            Some(Err(err)) => info!("err: {}", err),
            _ => {
                count += 1;
                if count % 5_000_000 == 0 {
                    info!("next is none")
                }
            }
        }
    }

    info!("End");

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    println!("Hello");
    info!("Hello");

    // let total = get_case_count().await.expect("Error getting case count");

    if let Err(e) = run_bnce().await {
        match e {
            // Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Testcase failed: {}", err),
        }
    }

    // update_reports().await.expect("Error updating reports");
}
