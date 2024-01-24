#![allow(non_snake_case)]
use futures_util::{SinkExt, StreamExt};
use regex::Regex;
use serde::*;
use std::time::{Duration, Instant};
use tokio::fs::{self};
use tokio::io::{self, AsyncWriteExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, WebSocketStream};

/// Average Price of the data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AveragePrice {
    pub average_price: f64,
}

/// For Sending the Data Request
#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub op: String,
    pub args: Vec<String>,
}

/// This is for the First Response you will response after
/// successful sent message.
#[derive(Serialize, Deserialize, Debug)]
pub struct SuccessResponse {
    pub success: bool,
    pub ret_msg: Option<String>,
    pub conn_id: String,
    pub request: Request,
}

/// This is for the Response you will start to get after SuccessResponse,
/// this actually contains the USD data
#[derive(Serialize, Deserialize, Debug)]
pub struct CryptoData {
    pub topic: String,
    pub data: Vec<Data>,
}

/// This for the Data Structure inside the CrytoData
#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub trade_time_ms: i64,
    pub timestamp: String,
    pub symbol: String,
    pub side: String,
    pub size: i64,
    pub price: f64,
    pub tick_direction: String,
    pub trade_id: String,
    pub cross_seq: i64,
    pub is_block_trade: String,
}

/// This is the FinalData after all the operations
#[derive(Serialize, Deserialize, Debug)]
pub struct FinalData {
    Data: Vec<CryptoData>,
    Average: Vec<AveragePrice>,
}

/// This Function does the work of connectiong to the server, sending the subribe request
/// for a symbol and listening for the actual data for specified duration. Last it returns
/// the average price and saves the data to a JSON file.
pub async fn process_websocket_data(
    times: u64,
    client_id: usize,
) -> Result<AveragePrice, Box<dyn std::error::Error>> {
    let url = url::Url::parse("wss://stream.bybit.com/realtime").expect("Failed to parse URL");
    let symbol = "BTCUSD";
    let file_path = format!("client_{}_data.json", client_id);
    let mut socket = connect_to_websocket(&url).await;
    let _ = subscribe_to_symbol(&mut socket, symbol).await;
    let (actual_data_responses, average_price_data) =
        receive_actual_data_responses(times, &mut socket).await;
    let temp = average_price_data.clone();
    let final_data = FinalData {
        Data: actual_data_responses,
        Average: vec![average_price_data],
    };
    write_to_file(&final_data, &file_path)
        .await
        .expect("Unable to write to file");
    log::info!(
        "Client {}. Cached Complete. The average USD price of BTC is: {}",
        client_id,
        temp.average_price
    );

    // append_to_file(&final_data, file_path)
    //     .await
    //     .expect("Unable to append to file");
    Ok(temp)
}

/// For connect to websocket
pub async fn connect_to_websocket(
    url: &url::Url,
) -> WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let (socket, _) = connect_async(url.clone()).await.expect("Failed to connect");
    socket
}

/// For sending the request to socekt
pub async fn subscribe_to_symbol(
    socket: &mut WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    symbol: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = Request {
        op: "subscribe".to_string(),
        args: vec![format!("trade.{}", symbol)],
    };
    let json_payload = serde_json::to_string(&payload).expect("Failed to serialize payload");
    let msg = Message::Text(json_payload);

    socket.send(msg).await.expect("Failed to send message");
    Ok(())
}

/// This is for parsing the data and performing the neccesary average operstions on it
pub async fn receive_actual_data_responses(
    times: u64,
    socket: &mut WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
) -> (Vec<CryptoData>, AveragePrice) {
    let mut actual_data_responses = Vec::new();
    let start_time = Instant::now();
    let mut total_price: f64 = 0.0;
    let mut total_count: usize = 0;

    if let Some(Ok(msg)) = socket.next().await {
        if let Ok(_success_response) =
            serde_json::from_str::<SuccessResponse>(msg.to_text().unwrap_or_default())
        {
            log::info!("Successfully recieved the response");
        } else {
            log::error!("Failed to parse the initial success response: {:?}", msg);
        }
    }

    while Instant::now() - start_time < Duration::from_secs(times) {
        if let Some(Ok(msg)) = socket.next().await {
            if let Ok(actual_data) =
                serde_json::from_str::<CryptoData>(msg.to_text().unwrap_or_default())
            {
                total_price += actual_data.data.iter().map(|data| data.price).sum::<f64>();
                total_count += actual_data.data.len();
                actual_data_responses.push(actual_data);
            } else {
                log::error!("Failed to parse the message: {:?}", msg);
            }
        }
    }
    let average_price = total_price / total_count as f64;
    let average_price_data = AveragePrice { average_price };
    (actual_data_responses, average_price_data)
}

/// Writes to a JSON file
pub async fn write_to_file(data: &FinalData, file_path: &str) -> io::Result<()> {
    let serialized = serde_json::to_string_pretty(data).unwrap_or_default();
    fs::write(file_path, serialized).await?;
    Ok(())
}

/// Appends to a JSON file
pub async fn append_to_file(data: &FinalData, file_path: &str) -> io::Result<()> {
    let serialized = serde_json::to_string_pretty(data).expect("Unable to serialize the data");

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(file_path)
        .await
        .expect("Unable to open the file");

    let check_empty = file
        .metadata()
        .await
        .map(|meta| meta.len() == 0)
        .unwrap_or(false);
    if !check_empty {
        file.write_all(b",").await.expect("Unable to write to file");
    }

    file.write_all(serialized.as_bytes())
        .await
        .expect("Unable to write to file");

    file.sync_all().await.expect("Failed to sync file");

    Ok(())
}

/// Reads the data from all the JSON Files
pub async fn read_from_file() -> io::Result<Vec<String>> {
    let file_pattern = r"^client_(\d+)_data\.json$";
    let regex_pattern =
        Regex::new(file_pattern).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    let mut all_contents = Vec::new();
    // let file_path = format!("client_{}_data.json", 0);
    let current_dir = std::env::current_dir()?.join(".");

    for entry in current_dir.read_dir()? {
        let entry = entry?;

        if let Some(file_name) = entry.file_name().to_str() {
            if regex_pattern.is_match(file_name) {
                let file_path = entry.path().to_str().unwrap().to_string();

                let content = fs::read_to_string(&file_path).await?;
                let trimmed_content = content.trim();

                if trimmed_content.is_empty() {
                    log::info!("File {} is empty", file_path);
                    // return Err(io::Error::new(io::ErrorKind::InvalidData, "File is empty"));
                } else {
                    log::info!("Content of {}: {}", file_path, trimmed_content);
                    all_contents.push(trimmed_content.to_string());
                }
            }
        }
    }

    // let content = tokio::fs::read_to_string(file_path.clone())
    //     .await
    //     .expect("Failed to read to the file");
    // let trimmed_content = content.trim();
    // if trimmed_content.is_empty() {
    //     log::info!("File {} is empty", file_path);
    //     // return Err(io::Error::new(io::ErrorKind::InvalidData, "File is empty"));
    // } else {
    //     log::info!("Content of {}: {}", file_path, trimmed_content);
    //     all_contents.push(trimmed_content.to_string());
    // }
    Ok(all_contents)
}
