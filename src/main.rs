/// Read list of domain names from the command line or a file
extern crate clap;
extern crate futures;

use clap::{App, Arg};
use futures::{stream, StreamExt};
use reqwest::Client;
use std::io::{self, BufRead};
use std::time::Duration;

/// A port number for the probe
type Port = u16;

/// The two possible protocols for a probe
#[derive(Debug, Copy, Clone)]
enum Protocol {
    Http,
    Https,
}

/// A probe is composed of a probe and a protocol.
#[derive(Debug, Copy, Clone)]
struct Probe {
    protocol: Protocol,
    port: Port,
}

impl Probe {
    /// Create a new probe from a protocol and port
    fn new(protocol: Protocol, port: Port) -> Probe {
        Probe { protocol, port }
    }

    /// Create a new http probe for the port.
    fn new_http(port: Port) -> Probe {
        Probe::new(Protocol::Http, port)
    }

    /// Create a new https probe for the port.
    fn new_https(port: Port) -> Probe {
        Probe::new(Protocol::Https, port)
    }

    /// Returns true if the port is the default for the protocol.
    fn is_default_port(&self) -> bool {
        match self {
            Probe {
                protocol: Protocol::Http,
                port: 80,
            } => true,
            Probe {
                protocol: Protocol::Https,
                port: 443,
            } => true,
            _ => false,
        }
    }
}

#[tokio::main]
async fn main() {
    let defatul_probes: Vec<Probe> = vec![Probe::new_http(80), Probe::new_https(443)];

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
    let timeout = command.value_of("timeout").unwrap();
    let concurrency = command.value_of("concurrency").unwrap();

    let concurrency_amount = match concurrency.parse::<usize>() {
        Ok(c) => c,
        Err(_e) => {
            println!(
                "-c --concurrency parameter was not a integer: {}",
                concurrency
            );
            std::process::exit(1);
        }
    };

    let (mut probes, errors) = match probe_args {
        Some(p) => parse_probes(p),
        None => (vec![], vec![]),
    };

    if !errors.is_empty() {
        println!("Invalid Probe arguments {:?}", errors);
        std::process::exit(1);
    }

    let timeout_duration = match timeout.parse::<u64>().map(Duration::from_millis) {
        Ok(t) => t,
        Err(_) => {
            println!("-t --timeout parameter was not a number: {}", timeout);
            std::process::exit(1);
        }
    };

    if run_default {
        probes.extend_from_slice(&defatul_probes)
    }

    let client = Client::builder()
        .connect_timeout(timeout_duration)
        .build()
        .unwrap();

    let stdin = io::stdin();
    stream::iter(stdin.lock().lines())
        .flat_map(|line| {
            let line = line.unwrap();
            stream::iter(&probes).map(move |probe| probe_to_url(&line, probe))
        })
        .map(|line| {
            let client = &client;
            async move { client.get(&line).send().await.map(|r| (line, r)) }
        })
        .buffer_unordered(concurrency_amount)
        .for_each(|b| async {
            match b {
                Ok((r, _res)) => println!("{:?}", r),
                Err(e) => eprintln!("Got an error: {}", e),
            }
        })
        .await;
}

fn probe_to_url(host: &str, probe: &Probe) -> String {
    match probe.protocol {
        Protocol::Http if probe.is_default_port() => format!("http://{}", host),
        Protocol::Http => format!("http://{}:{}", host, probe.port),
        Protocol::Https if probe.is_default_port() => format!("https://{}", host),
        Protocol::Https => format!("https://{}:{}", host, probe.port),
    }
}

/// Default is to use http:80 and https:443
fn parse_probes(probes: Vec<&str>) -> (Vec<Probe>, Vec<String>) {
    let (probes, errors): (Vec<_>, Vec<_>) = probes
        .iter()
        .map(|p| {
            let parts: Vec<&str> = p.split(':').collect();
            if parts.len() == 2 {
                match parts[1].parse::<u16>() {
                    Ok(port) if parts[0] == "http" => Ok(Probe::new_http(port)),
                    Ok(port) if parts[0] == "https" => Ok(Probe::new_https(port)),
                    _ => Err(format!("Error parsing probe: {}", p)),
                }
            } else {
                Err(format!("Error parsing probe: {}", p))
            }
        })
        .partition(Result::is_ok);
    let probes: Vec<_> = probes.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
    (probes, errors)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_probe_to_url_default_http() {
        assert_eq!(
            probe_to_url("demo.com", &Probe::new_http(80)),
            "http://demo.com"
        );
    }

    #[test]
    fn test_probe_to_url_default_https() {
        assert_eq!(
            probe_to_url("demo.com", &Probe::new_https(443)),
            "https://demo.com"
        );
    }
}
