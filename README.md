# hprobe [![CircleCI](https://circleci.com/gh/oliverdaff/hprobe.svg?style=shield)](https://circleci.com/gh/oliverdaff/hprobe)

Takes a list of domains and probes for working http and http services.

## Installation
While this library is in initial state of development installation is done using cargo.

```bash
git checkout https://github.com/oliverdaff/hprobe
cargo test 
cargo install --path .
```

## Usage

### Basic

Reads a list of domains from stdin.

```bash
cat domains.txt
domain1.com
domain2.com
domain3.com

cat domains.tx | hprobe
http://domain1.com
http://domain2.com
http://domain3.com
https://domain1.com
https://domain2.com
https://domain3.com
```

### Flags And Options
```bash
hprobe --help

hprobe 0.1
A fast http probe

USAGE:
    hprobe [FLAGS] [OPTIONS]

FLAGS:
    -k, --insecure            Accept invalid certificates.
    -h, --help                Prints help information
    -s, --suppress_default    do not process the default http and https ports
    -V, --version             Prints version information

OPTIONS:
    -c, --concurrency <CONCURRENCY>    The number of concurrent requests [default: 20]
    -p, --probe <PROBE>...             protocol port pair <http|https>:<port>
        --proxy-all <PROXY_ALL>        The url of the proxy to for all requests.
        --proxy-http <PROXY_HTTP>      The url of the proxy to for http requests.
        --proxy-https <PROXY_HTTPS>    The url of the proxy to for https requests.
    -t, --timeout <TIMEOUT>            The timeout for the connect phase (ms) [default: 1000]
    -u, --user_agent <user_agent>      Set the requests USER-AGENT header
```

### Proxies
Hprobe will look in environment variables to set HTTP or HTTPS proxies.

`HTTP_PROXY` or `http_proxy` provide http proxies for http connections while `HTTPS_PROXY` or `https_proxy` provide HTTPS proxies for HTTPS connections.

The `--proxy-all` flag can be used to proxy all requests on the command line.
The `--proxy-http` flag can be used to proxy all http requests on the command line, but can not be used with `--proxy-all`.
The `--proxy-https` flag can be used to proxy all https requests on the command line, but can not be used with `--proxy-all`.

### Invalid Certificates
`-k, --insecure` using either of these two flags means any certificate for any site will be trusted for use. This includes expired certificates.

### Headers
`-u --user-agent` set the user agent for the request.

## Tests
The tests can be invoked with `cargo test`.

## Credits
This project was inspired by [httprobe](https://github.com/tomnomnom/httprobe) written in golang.

## License
MIT © Oliver Daff