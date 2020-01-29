#[cfg(feature = "update")]
use clap::ArgMatches;
use clap::{crate_authors, crate_version, App, Arg, ArgGroup, SubCommand};
use ipnetwork::Ipv4Network;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;

extern crate ipset_lookup;
use crate::ipset_lookup::lookup::LookupSets;

#[cfg(feature = "bench")]
use crate::ipset_lookup::lookup::test_speed;

fn app_params<'a, 'b>() -> App<'a, 'b> {
    #[allow(unused_mut)]
    let mut app = App::new("ipset-lookup")
    .about("Fast lookup through ipset data")
    .version(crate_version!())
    .author(crate_authors!())
    .arg(Arg::with_name("glob").group("input")
        .long("glob")
        .short("g")
        .takes_value(true)
        .empty_values(false)
        .global(true)
        .help("input ipset/netset files, glob syntax (defaults to: blocklist-ipsets/**/*.*set)"))
    .subcommand(SubCommand::with_name("lookup")
        .about("run a lookup")
        .group(ArgGroup::with_name("input"))
        .group(ArgGroup::with_name("find").multiple(true).required(true))
        .group(ArgGroup::with_name("output"))
        .arg(Arg::with_name("file").group("find")
            .long("file")
            .short("f")
            .takes_value(true)
            .multiple(true)
            .empty_values(false)
            .help("compare to a list of IPs in a file"))
        .arg(Arg::with_name("ip").group("find")
            .long("ip")
            .short("i")
            .takes_value(true)
            .multiple(true)
            .empty_values(false)
            .help("compare to an IP"))
        .arg(Arg::with_name("net").group("find")
            .long("net")
            .short("n")
            .takes_value(true)
            .multiple(true)
            .empty_values(false)
            .help("compare to a net")));

    #[cfg(feature = "bench")]
    {
        app = app.subcommand(SubCommand::with_name("bench").about("run a quick benchmark"));
    }

    #[cfg(feature = "update")]
    {
        app = app.subcommand(SubCommand::with_name("update").about("update ipsets"));
    }

    app
}

#[cfg(feature = "update")]
fn update_command(_: &ArgMatches) {
    use git2::Repository;

    let url = "https://github.com/firehol/blocklist-ipsets.git";
    let repo = match Repository::open("blocklist-ipsets") {
        Ok(repo) => repo,
        Err(e1) => match Repository::clone(url, "blocklist-ipsets") {
            Ok(repo) => repo,
            Err(e2) => panic!("failed to clone: {} and {}", e1, e2),
        },
    };

    match repo.checkout_head(None) {
        Ok(_) => {}
        Err(e) => panic!("git checkout error: {}", e),
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = app_params();

    let m = app.get_matches();
    //    println!("{:?}", m);

    let globfiles = if m.is_present("glob") {
        m.value_of("glob").unwrap()
    } else {
        "blocklist-ipsets/**/*.*set"
    };

    match m.subcommand() {
        ("lookup", Some(sub_m)) => {
            let ipsets = LookupSets::new(globfiles);

            if sub_m.is_present("file") {
                let files: Vec<_> = sub_m.values_of("file").unwrap().collect();
                for path in files {
                    let file = File::open(path)?;
                    let buffered = BufReader::new(file);
                    let data = buffered
                        .lines()
                        .map(|l| l.unwrap())
                        .filter(|l| !l.starts_with('#'))
                        .map(|l| l.parse().expect("invalid ip"));
                    for ip in data {
                        let result = ipsets.lookup_by_ip(ip);
                        println!(r#"{{"ip":"{}", feeds:{:?}}}"#, ip, result);
                    }
                }
            }
            if sub_m.is_present("ip") {
                let ips: Vec<_> = sub_m.values_of("ip").unwrap().collect();
                let ips: Vec<Ipv4Addr> = ips
                    .iter()
                    .map(|ip| ip.parse().expect("invalid ip address"))
                    .collect();
                for ip in ips {
                    let result = ipsets.lookup_by_ip(ip);
                    println!(r#"{{"ip":"{}", feeds:{:?}}}"#, ip, result);
                }
            }
            if sub_m.is_present("net") {
                let nets: Vec<_> = sub_m.values_of("net").unwrap().collect();
                let nets: Vec<Ipv4Network> = nets
                    .iter()
                    .map(|ip| ip.parse().expect("invalid net"))
                    .collect();
                for net in nets {
                    let result = ipsets.lookup_by_net(net);
                    println!(r#"{{"ip":"{}", feeds:{:?}}}"#, net, result);
                }
            }
        }
        #[cfg(feature = "bench")]
        ("bench", Some(_)) => test_speed(globfiles),
        #[cfg(feature = "update")]
        ("update", Some(sub_m)) => update_command(sub_m),
        _ => {}
    }
    Ok(())
}
