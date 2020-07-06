/// Read list of domain names from the command line or a file
/// Read a list of probes
/// protocol:port from the command line using the -p flag
/// Use -nd flag to specify no defaults
/// Timeout -t
/// Concurrency -c
///
/// Future add socks and alternate dns support
extern crate clap;
extern crate futures;

use clap::{App, Arg};
use futures::{stream, StreamExt};
use reqwest::Client;
use std::io::{self, BufRead};

#[tokio::main]
async fn main() {
    let command = App::new("hprobe")
        .version("0.1")
        .about("A fast http probe")
        .arg(
            Arg::with_name("probes")
                .short("p")
                .long("probe")
                .value_name("PROBE")
                .help("protocol port pair <http|https>:<port>")
                .takes_value(true)
                .multiple(true)
                .required(false),
        )
        .arg(
            Arg::with_name("suppress_default")
                .short("s")
                .long("suppress_default")
                .value_name("SUPPRESS")
                .help("do not process the default http and https ports")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .value_name("TIMEOUT")
                .help("The timeout for the connect phase (ms)")
                .takes_value(true)
                .required(false)
                .default_value("1000"),
        )
        .arg(
            Arg::with_name("concurrency")
                .short("c")
                .long("concurrency")
                .value_name("CONCURRENCY")
                .help("The number of concurrent requests")
                .takes_value(true)
                .required(false)
                .default_value("20"),
        )
        .get_matches();

    let probes: Option<Vec<_>> = command.values_of("probes").map(|x| x.collect());
    let run_default = !command.is_present("suppress_default");
    let timeout = command.value_of("timeout").unwrap().parse::<u32>();
    let concurrency = command.value_of("concurrency").unwrap().parse::<u32>();

    println!(
        "probes {:?}, run default {:?}, timeout {:?}, concurrency {:?}",
        probes, run_default, timeout, concurrency
    );

    let client = Client::builder().build().unwrap();

    let stdin = io::stdin();
    let result = stream::iter(stdin.lock().lines())
        .map(|line| {
            let line = line.unwrap();
            let client = &client;
            async move { client.get(&line).send().await.map(|r| (line, r)) }
        })
        .buffer_unordered(2);

    result
        .for_each(|b| async {
            match b {
                Ok((r, _res)) => println!("{:?}", r),
                Err(e) => eprintln!("Got an error: {}", e),
            }
        })
        .await;
}
