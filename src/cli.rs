use simple::process_websocket_data;
use simple::read_from_file;
use std::clone::Clone;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use crate::aggregator::Aggregator;
use clap::{App, Arg};

/// For Parsing CLI args and executing the associated command
pub async fn parse_args() {
    let matches = App::new("Simple CLI App")
        .version("1.0")
        .author("Your Name")
        .about("A simple CLI app with different modes")
        .arg(
            Arg::with_name("mode")
                .short("m")
                .long("mode")
                .value_name("MODE")
                .help("Sets the mode")
                .takes_value(true)
                .required(true)
                .possible_values(&["cache", "read"]),
        )
        .arg(
            Arg::with_name("times")
                .short("t")
                .long("times")
                .value_name("TIMES")
                .help("Number of times for processing websocket data (required for 'cache' mode)")
                .takes_value(true)
                .required_if("mode", "cache")
                .validator(|v| v.parse::<u64>().map(|_| ()).map_err(|e| e.to_string())),
        )
        .arg(
            Arg::with_name("clients")
                .short("c")
                .long("client")
                .value_name("CLIENT")
                .takes_value(true)
                .default_value("1")
                .help("To Instantiate additional clients asynchronously"),
        )
        .get_matches();

    match matches.value_of("mode").unwrap() {
        "cache" => {
            let times = matches.value_of("times").unwrap().parse::<u64>().unwrap();
            let client_number = matches.value_of("clients").unwrap().parse::<u64>().unwrap();
            let instantiate_extra_clients = matches.is_present("clients");

            let aggregator = Arc::new(Mutex::new(Aggregator::new()));

            let num_clients = if instantiate_extra_clients {
                client_number.max(1)
            } else {
                1
            };

            let client_tasks = (0..num_clients).map(|i| {
                let aggregator = Arc::clone(&aggregator);
                tokio::spawn(client_process(i as usize, times, aggregator))
            });

            futures_util::future::try_join_all(client_tasks)
                .await
                .expect("Failed to wait");

            let fin_avg = aggregator.lock().unwrap().final_average();
            log::info!("Final Average: {}", fin_avg)

            // let _ = process_websocket_data(times).await;
        }
        "read" => {
            let _ = read_from_file().await;
        }
        _ => unreachable!(),
    }
}

/// For proccessing Websocket data for a client asyncly and sending it to the validator.
async fn client_process(client_id: usize, times: u64, aggregator: Arc<Mutex<Aggregator>>) {
    let start_time = Instant::now() + Duration::from_secs(1);

    let client_start_time = start_time + Duration::from_secs(client_id as u64);

    tokio::time::sleep(client_start_time - Instant::now()).await;

    let result = process_websocket_data(times, client_id).await;
    if let Ok(average) = result {
        aggregator.lock().unwrap().add_average(average);
    }
}
