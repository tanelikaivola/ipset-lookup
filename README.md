# ipset-lookup

ipset is a command-line tool that takes networks or IPs and searches through a lot of different threat feeds quickly.
It can also download the feed data necessary to perform the queries.

ipset_lookup includes the same functionality as a library.

[![Crates.io](https://img.shields.io/crates/v/ipset_lookup)](https://crates.io/crates/ipset_lookup)
[![Linux build Status](https://travis-ci.org/tanelikaivola/ipset-lookup.svg)](https://travis-ci.org/tanelikaivola/ipset-lookup)

### Documentation quick links

* [User Guide](#user-guide)
* [Installation](#installation)
* [Building](#building)

## User Guide

### Quickstart

```
$ ipset update
$ ipset lookup -i 8.8.8.8
{"ip":"8.8.8.8", "feeds":["abuse/firehol_abusers_30d", "abuse/stopforumspam", "abuse/stopforumspam_180d", "abuse/stopforumspam_365d", "abuse/stopforumspam_90d", "geolocation/continent_na", "geolocation/country_us", "geolocation/id_continent_na", "geolocation/id_country_us", "geolocation/ip2location_continent_na", "geolocation/ip2location_country_us", "geolocation/ipip_country_anycast", "malware/hphosts_emd", "organizations/coinbl_hosts", "organizations/hphosts_ats", "reputation/hphosts_fsa", "reputation/hphosts_psh", "reputation/packetmail_emerging_ips"]}
$ ipset lookup -n 127.0.0.0/8
{"ip":"127.0.0.0/8", "feeds":["abuse/botscout_30d", "abuse/botscout_7d", "abuse/hphosts_hfs", "attacks/firehol_level1", "attacks/firehol_level4", "geolocation/ip2location_country_countryless", "malware/hphosts_emd", "malware/hphosts_exp", "malware/hphosts_hjk", "organizations/coinbl_hosts", "organizations/coinbl_hosts_browser", "organizations/hphosts_ats", "reputation/hphosts_fsa", "reputation/hphosts_mmt", "reputation/hphosts_pha", "reputation/hphosts_psh", "reputation/hphosts_wrz", "reputation/nullsecure", "spam/cleanmx_phishing", "spam/hphosts_grm", "spam/lashback_ubl", "unroutable/cidr_report_bogons", "unroutable/iblocklist_cidr_report_bogons"]}
```

### ipset lookup

Main functionality is to query threat feed data (`ipset lookup`) that is stored locally (and downloaded by `ipset update`).

Queries use glob-patterns as an input to read feed data and that data can be queried by ip, network (CIDR) or list of IPs in a file.

```
USAGE:
    ipset.exe lookup [OPTIONS] <--file <file>...|--ip <ip>...|--net <net>...>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --file <file>...    compare to a list of IPs in a file
    -g, --glob <glob>       input ipset/netset files, glob syntax (defaults to: blocklist-ipsets/**/*.*set)
    -i, --ip <ip>...        compare to an IP
    -n, --net <net>...      compare to a net
```

### ipset-zmq

ZeroMQ microservice for serving threat feed data that is stored locally (and downloaded by `ipset update`).

Queries use glob-patterns as an input to read feed data and that data can be queried by ip, network (CIDR) or list of IPs in a file.

Currently binds to hard-coded tcp://127.0.0.1:5555 for ZeroMQ ROUTER.

```
USAGE:
    ipset-zmq [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -g, --glob <glob>    input ipset/netset files, glob syntax (defaults to: blocklist-ipsets/**/*.*set)
```

## Installation

The main binary name for ipset-lookup is `ipset`. ZeroMQ microservice executable is `ipset-zmq` (if enabled).

No prebuild binaries are available currently.

Use one of:
```
cargo install ipset_lookup --locked
cargo install --all-features ipset_lookup --locked
```

## Building

[![Build Status](https://travis-ci.org/tanelikaivola/ipset-lookup.svg)](https://travis-ci.org/tanelikaivola/ipset-lookup)

Multiple different configurations can be build.

`--all-features` and `--features microservice` require libzmq which might not be available.

Try one of the following build commands:

```
cargo build --release --all-features
cargo build --release --features windows-all
cargo build --release --features microservice
```

## Optional features

ipset-lookup has some optional features

- `update`: provides update subcommand which is a git client that can update feeds (currently https://github.com/firehol/blocklist-ipsets/)
- `bench`: provides bench subcommand which includes benchmarking functionality
- `microservice`: provides ipset-zmq microservice executable
- `vendored-zmq`: zmq is provided as vendored build
- `windows-all`: compiles all the parts that are likely to compile on a Windows machine
