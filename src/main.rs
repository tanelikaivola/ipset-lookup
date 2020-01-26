use std::net::Ipv4Addr;
use ipnetwork::Ipv4Network;
use clap::{Arg, ArgGroup, App, SubCommand, crate_version, crate_authors};

mod lookup;
use crate::lookup::LookupSets;

fn app_params<'a,'b>() -> App<'a, 'b> {
    App::new("ipset-lookup")
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
            .help("compare to a net")))
    .subcommand(SubCommand::with_name("bench")
        .about("run a quick benchmark"))    
}

fn main() {
    let app = app_params();

    let m = app.get_matches();
    println!("{:?}", m);

    let globfiles;
    if m.is_present("glob") {
        globfiles = m.value_of("glob").unwrap();
    } else {
        globfiles = "blocklist-ipsets/**/*.*set";
    }

    match m.subcommand() {
        ("lookup",  Some(sub_m)) => {
            let ipsets = LookupSets::new(globfiles);

            if sub_m.is_present("file") {
                let files: Vec<_> = sub_m.values_of("file").unwrap().collect();
                unimplemented!("file handling not implemented");
            }
            if sub_m.is_present("ip") {
                let ips: Vec<_> = sub_m.values_of("ip").unwrap().collect();
                let ips: Vec<Ipv4Addr> = ips.iter().map(|ip| ip.parse().expect("invalid ip address")).collect();
                for ip in ips {
                    let result = ipsets.lookup_by_ip(&ip);
                    println!("{} {:?}", ip, result);
                }
            }
            if sub_m.is_present("net") {
                let nets: Vec<_> = sub_m.values_of("net").unwrap().collect();
                let nets: Vec<Ipv4Network> = nets.iter().map(|ip| ip.parse().expect("invalid net")).collect();
                for net in nets {
                    let result = ipsets.lookup_by_net(&net);
                    println!("{} {:?}", net, result);
                }
            }
        },
        ("bench",   Some(_)) => {lookup::test_speed(globfiles)},
        _                       => {},
    }
}