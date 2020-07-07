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
use regex::Regex;

use clap::{App, Arg};
use futures::{stream, StreamExt};
use lazy_static::lazy_static;
use reqwest::Client;
use std::io::{self, BufRead};
use std::time::Duration;

type Port = u16;
#[derive(Debug, Copy, Clone)]
enum Protocol {
    Http,
    Https,
}

#[derive(Debug, Copy, Clone)]
struct Probe {
    port: Port,
    protocol: Protocol,
}

#[tokio::main]
async fn main() {
    let defatul_probes: Vec<Probe> = vec![
        Probe {
            protocol: Protocol::Http,
            port: 80,
        },
        Probe {
            protocol: Protocol::Https,
            port: 443,
        },
    ];

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

    let probe_args: Option<Vec<_>> = command.values_of("probes").map(|x| x.collect());
    let run_default = !command.is_present("suppress_default");
    let timeout = command.value_of("timeout").unwrap().parse::<u32>();
    let concurrency = command.value_of("concurrency").unwrap().parse::<u32>();

    println!(
        "probes {:?}, run default {:?}, timeout {:?}, concurrency {:?}",
        probe_args, run_default, timeout, concurrency
    );

    let (mut probes, errors) = match probe_args {
        Some(p) => parse_probes(p),
        None => (vec![], vec![]),
    };

    if !errors.is_empty() {
        println!("Invalid Probe arguments {:?}", errors);
        return;
    }

    if run_default {
        probes.extend_from_slice(&defatul_probes)
    }

    let client = Client::builder().timeout(Duration::from_secs(1)).build().unwrap();

    let stdin = io::stdin();
    let result = stream::iter(stdin.lock().lines())
        .flat_map(|line| {
            let probes = &probes;
            let line = line.unwrap();
            stream::iter(probes).map(move |probe| probe_to_url(&line, probe))
        })
        .map(|line| {
            let line = line;
            let client = &client;
            async move { client.get(&line).send().await.map(|r| (line, r)) }
        })
        .buffer_unordered(10);

    result
        .for_each(|b| async {
            match b {
                Ok((r, _res)) => println!("{:?}", r),
                Err(e) => eprintln!("Got an error: {}", e),
            }
        })
        .await;
}

fn probe_to_url(host: &str, probe: &Probe) -> String {
    match probe {
        Probe {
            protocol: Protocol::Http,
            port,
        } if port == &80 => format!("http://{}", host),
        Probe {
            protocol: Protocol::Http,
            port,
        } => format!("http://{}:{}", host, port),
        Probe {
            protocol: Protocol::Https,
            port,
        } if port == &443 => format!("https://{}", host),
        Probe {
            protocol: Protocol::Https,
            port,
        } => format!("https://{}:{}", host, port),
    }
}

/// Default is to use http:80 and https:443
fn parse_probes(probes: Vec<&str>) -> (Vec<Probe>, Vec<String>) {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(http|https):\b([1-9]\d{0,3}|[1-6][0-5][0-5][0-3][0-5])\b"
        )
        .unwrap();
    }
    let (probes, errors): (Vec<_>, Vec<_>) = probes
        .iter()
        .map(|p| match RE.captures(p) {
            Some(cap) => {
                let groups = (cap.get(1), cap.get(2));
                println!("{:?}", groups);
                match groups {
                    (Some(prot), Some(port)) if prot.as_str() == "http" => Ok(Probe {
                        protocol: Protocol::Http,
                        port: port.as_str().parse().unwrap(),
                    }),
                    (Some(prot), Some(port)) if prot.as_str() == "https" => Ok(Probe {
                        protocol: Protocol::Https,
                        port: port.as_str().parse().unwrap(),
                    }),
                    _ => Err(format!("Error parsing probe: {}", p)),
                }
            }
            None => {
                println!("{:?}", RE.captures(p));
                Err(format!("Error parsing probe: {}", p))
            }

        })
        .partition(Result::is_ok);
    let probes: Vec<_> = probes.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
    (probes, errors)
}
