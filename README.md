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

A fast http probe

USAGE:
    hprobe [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                Prints help information
    -s, --suppress_default    do not process the default http and https ports
    -V, --version             Prints version information

OPTIONS:
    -c, --concurrency <CONCURRENCY>    The number of concurrent requests [default: 20]
    -p, --probe <PROBE>...             protocol port pair <http|https>:<port>
    -t, --timeout <TIMEOUT>            The timeout for the connect phase (ms) [default: 1000]
```

## Tests
The tests can be invoked with `cargo test`.

## Credits
This project was inspired by [httprobe](https://github.com/tomnomnom/httprobe) written in golang.

## License
MIT © Oliver Daff