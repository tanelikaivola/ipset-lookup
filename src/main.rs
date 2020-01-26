
use glob::glob;
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use ipnetwork::Ipv4Network;
use std::iter::FromIterator;
use rayon::prelude::*;
use clap::{Arg, ArgGroup, App, SubCommand, crate_version, crate_authors, ArgMatches};

trait Lookup {
    fn lookup_by_ip(&self, ip:&Ipv4Addr) -> bool;
}

impl Lookup for Vec<Ipv4Network> {
    fn lookup_by_ip(&self, ip:&Ipv4Addr) -> bool {
        self.iter().any(|net| net.contains(*ip))
    }
}

impl Lookup for Vec<Ipv4Addr> {
    fn lookup_by_ip(&self, ip:&Ipv4Addr) -> bool {
        self.iter().any(|other| other == ip)
    }
}

fn glob_vec(pattern: &str) -> Vec<PathBuf> {
    glob(pattern).unwrap().map(|r| r.unwrap()).collect()
}

fn parse_file(path : &std::path::PathBuf) -> (String, Vec<Ipv4Network>) {
    let stem = path.file_stem().unwrap().to_str().unwrap().to_string();

    let file = File::open(path).unwrap();
    let buffered = BufReader::new(file);
    let data : Vec<Ipv4Network> = buffered.lines().map(
        |l| l.unwrap()
    ).filter(
        |l| ! l.starts_with("#")
    ).map(
        |l| l.parse().unwrap()
    ).collect();
    
    (stem, data)
}

struct LookupSets {
    data : HashMap<String,Vec<Ipv4Network>>
}

impl LookupSets {
    fn new(glob : &str) -> LookupSets {
        let files = glob_vec(glob);

        let ipsetiter : Vec<_> = files.par_iter().map(
            |path| parse_file(&path.to_path_buf())
        ).collect();
        LookupSets {data: HashMap::from_iter(ipsetiter)}
    }
    fn lookup_by_ip(&self, ip:&Ipv4Addr) -> Vec<&String> {
        let mut output = self.data.par_iter().filter(
            |(_, nets)| nets.lookup_by_ip(ip)
        ).map(
            |(name, _)| name
        ).collect::<Vec<_>>();
        output.sort();
        output
    }
    fn lookup_by_str(&self, ip:&str) -> Vec<&String> {
        let ip : Ipv4Addr = ip.parse().expect("invalid ip address");
        self.lookup_by_ip(&ip)
    }
    fn lookup_by_net(&self, other:&Ipv4Network) -> Vec<&String> {
        let mut output = self.data.par_iter().filter(
            |(_, nets)| nets.iter().any(|net| net.overlaps(*other))
        ).map(
            |(name, _)| name
        ).collect::<Vec<_>>();
        output.sort();
        output
    }
}

#[test]
fn test_sanity() {
    let ipsets = LookupSets::new("blocklist-ipsets/**/*.*set");
    let ip : Ipv4Addr = "8.8.8.8".parse().expect("Invalid IP");
    ipsets.lookup_by_ip(&ip);

    let mut rawips : Vec<Ipv4Addr> = Vec::new();
    rawips.insert(0, "8.8.8.8".parse().unwrap());
    let ip : Ipv4Addr = "8.8.8.8".parse().unwrap();
    assert!(rawips.lookup_by_ip(&ip), "lookup_by_ip is not eq");

    assert!(ipsets.lookup_by_str("8.8.8.8").len()>0, "lookup_by_ip is not eq");
}

fn test_speed(m : &ArgMatches) {
    let globfiles;
    if m.is_present("glob") {
        globfiles = m.value_of("glob").unwrap();
    } else {
        globfiles = "blocklist-ipsets/**/*.*set";
    }

    use std::time::Instant;
    let now = Instant::now();
    let ipsets = LookupSets::new(globfiles);
    println!("{:.3} s loading", now.elapsed().as_secs_f64());
    let categories = ipsets.lookup_by_net(&("0.0.0.0/0".parse().unwrap()));
    println!("Loaded {} categories", categories.len());

    
    let ip0 : Ipv4Addr = "0.0.0.0".parse().expect("Invalid IP");
    let net : Ipv4Network = "64.135.235.144/31".parse().expect("Invalid network");
    let net0 : Ipv4Network = "0.0.0.0/0".parse().expect("Invalid network");

    let now = Instant::now();
    let _x : Vec<_> = (1..100).map(|_x|
        ipsets.lookup_by_ip(&ip0)
    ).collect();
    println!("{:.3} ms / ip lookup", now.elapsed().as_secs_f64()/100.0*1000.0);

    let now = Instant::now();
    let _x : Vec<_> = (1..100).map(|_x|
        ipsets.lookup_by_net(&net)
    ).collect();
    println!("{:.3} ms / network lookup (maybe worst case)", now.elapsed().as_secs_f64()/100.0*1000.0);

    let now = Instant::now();
    let _x : Vec<_> = (1..100).map(|_x|
        ipsets.lookup_by_net(&net0)
    ).collect();
    println!("{:.3} ms / network lookup (best case)", now.elapsed().as_secs_f64()/100.0*1000.0);    
}

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
        ("bench",   Some(sub_m)) => {test_speed(&sub_m)},
        _                       => {},
    }

}