#![allow(dead_code)]

use glob::glob;
use ipnetwork::Ipv4Network;
use rayon::prelude::*;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use std::path::PathBuf;

use failure::Error;

pub trait Lookup {
    fn lookup_by_ip(&self, ip: Ipv4Addr) -> bool;
    fn lookup_by_net(&self, net: Ipv4Network) -> bool;
}

impl Lookup for Vec<Ipv4Network> {
    fn lookup_by_ip(&self, ip: Ipv4Addr) -> bool {
        self.iter().any(|net| net.contains(ip))
    }
    fn lookup_by_net(&self, other: Ipv4Network) -> bool {
        self.iter().any(|net| net.overlaps(other))
    }
}

impl Lookup for Vec<Ipv4Addr> {
    fn lookup_by_ip(&self, ip: Ipv4Addr) -> bool {
        self.iter().any(|other| *other == ip)
    }
    fn lookup_by_net(&self, net: Ipv4Network) -> bool {
        self.iter().any(|other| net.contains(*other))
    }
}

fn glob_vec(pattern: &str) -> Vec<PathBuf> {
    glob(pattern).unwrap().map(|r| r.unwrap()).collect()
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct NetSetFeed {
    category: String,
    name: String,
}
impl fmt::Debug for NetSetFeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#""{}/{}""#, self.category, self.name)
    }
}

#[derive(Clone)]
struct NetSet {
    feed: NetSetFeed,
    nets: Vec<Ipv4Network>,
    ips: Vec<Ipv4Addr>,
}
impl fmt::Debug for NetSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#""{}/{} ({} nets)""#,
            self.feed.category,
            self.feed.name,
            self.nets.len()
        )
    }
}

fn parse_file(path: &std::path::PathBuf) -> Result<NetSet, Error> {
    let name = path.file_stem().unwrap().to_str().unwrap().to_string();

    let file = File::open(path)?;
    let buffered = BufReader::new(file);
    let lines = buffered.lines();
    let comments: Vec<_> = lines
        .map(|l| l.unwrap())
        .take_while(|l| l.starts_with('#'))
        .filter(|l| l.starts_with("# Category        : "))
        .map(|l| l.replace("# Category        : ", ""))
        .collect();
    let category: String = if comments.len() == 1 {
        String::from(&comments[0])
    } else {
        //        println!("failed to find category {}", comments.len());
        String::from("other")
    };

    // reinitialize the reader (FIXME)
    let file = File::open(path)?;
    let buffered = BufReader::new(file);
    let lines = buffered
        .lines()
        .map(|l| l.unwrap())
        .filter(|l| !l.starts_with('#'));
    let (nets, ips): (Vec<_>, Vec<_>) = lines.partition(|l| l.contains('/'));

    let nets: Vec<Ipv4Network> = nets
        .iter()
        .map(|l| l.parse())
        .filter_map(|ip| ip.ok()) // TODO: errors ignored, collect statistics
        .collect();

    let ips: Vec<Ipv4Addr> = ips
        .iter()
        .map(|l| l.parse())
        .filter_map(|ip| ip.ok()) // TODO: errors ignored, collect statistics
        .collect();

    Ok(NetSet {
        feed: NetSetFeed { name, category },
        nets,
        ips,
    })
}

#[derive(Clone)]
pub struct LookupSets {
    data: Vec<NetSet>,
}

impl LookupSets {
    pub fn new(glob: &str) -> LookupSets {
        let files = glob_vec(glob);

        let (ipsetiter, errors): (Vec<_>, Vec<Result<_, Error>>) = files
            .par_iter()
            .map(|path| parse_file(&path.to_path_buf()))
            .partition(Result::is_ok); // TODO: errors ignored, collect statistics
        let ipsetiter: Vec<NetSet> = ipsetiter.into_iter().map(Result::unwrap).collect();
        let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
        if !errors.is_empty() {
            eprintln!("{:?}", errors);
        }
        LookupSets { data: ipsetiter }
    }
    pub fn lookup_by_ip(&self, ip: Ipv4Addr) -> Vec<&NetSetFeed> {
        let mut output: Vec<_> = self
            .data
            .par_iter()
            .filter(|netset| netset.nets.lookup_by_ip(ip) || netset.ips.lookup_by_ip(ip))
            .map(|netset| &netset.feed)
            .collect();
        output.sort();
        output
    }
    pub fn lookup_by_net(&self, other: Ipv4Network) -> Vec<&NetSetFeed> {
        let mut output: Vec<_> = self
            .data
            .par_iter()
            .filter(|netset| netset.nets.lookup_by_net(other) || netset.ips.lookup_by_net(other))
            .map(|netset| &netset.feed)
            .collect();
        output.sort();
        output
    }
}

#[test]
fn test_loading() {
    let ipsets = LookupSets::new("blocklist-ipsets/**/*.*set");
    let ip: Ipv4Addr = "8.8.8.8".parse().expect("Invalid IP");
    let categories: Vec<&NetSetFeed> = ipsets.lookup_by_ip(ip);
    assert!(!categories.is_empty(), "no results for a lookup");
    //    println!("{:?}", categories);
}

#[test]
fn test_lookups() {
    let mut ips: Vec<Ipv4Addr> = Vec::new();
    ips.insert(0, "8.8.8.8".parse().unwrap());
    let mut nets: Vec<Ipv4Network> = Vec::new();
    nets.insert(0, "8.8.8.8/8".parse().unwrap());
    let ip: Ipv4Addr = "8.8.8.8".parse().unwrap();
    let ip2: Ipv4Addr = "1.1.1.1".parse().unwrap();
    let net: Ipv4Network = "8.8.8.8/8".parse().unwrap();
    let net2: Ipv4Network = "1.1.1.1/8".parse().unwrap();

    assert!(ips.lookup_by_ip(ip), "ips - lookup_by_ip is not eq");
    assert!(!ips.lookup_by_ip(ip2), "ips - lookup_by_ip is not neq");
    assert!(ips.lookup_by_net(net), "ips - lookup_by_net is not eq");
    assert!(!ips.lookup_by_net(net2), "ips - lookup_by_net is not neq");

    assert!(nets.lookup_by_ip(ip), "nets - lookup_by_ip is not eq");
    assert!(nets.lookup_by_net(net), "nets - lookup_by_net is not eq");
    assert!(!nets.lookup_by_ip(ip2), "nets - lookup_by_ip is not neq");
    assert!(!nets.lookup_by_net(net2), "nets - lookup_by_net is not neq");
}

#[cfg(feature = "bench")]
pub fn test_speed(glob: &str) {
    use std::time::Instant;
    let now = Instant::now();
    let ipsets = LookupSets::new(glob);
    println!("{:.3} s loading", now.elapsed().as_secs_f64());
    let categories = ipsets.lookup_by_net("0.0.0.0/0".parse().unwrap());
    println!("Loaded {} categories", categories.len());

    let ip0: Ipv4Addr = "0.0.0.0".parse().expect("Invalid IP");
    let net: Ipv4Network = "64.135.235.144/31".parse().expect("Invalid network");
    let net0: Ipv4Network = "0.0.0.0/0".parse().expect("Invalid network");

    let now = Instant::now();
    let _x: Vec<_> = (1..100).map(|_x| ipsets.lookup_by_ip(ip0)).collect();
    println!(
        "{:.3} ms / ip lookup",
        now.elapsed().as_secs_f64() / 100.0 * 1000.0
    );

    let now = Instant::now();
    let _x: Vec<_> = (1..100).map(|_x| ipsets.lookup_by_net(net)).collect();
    println!(
        "{:.3} ms / network lookup (maybe worst case)",
        now.elapsed().as_secs_f64() / 100.0 * 1000.0
    );

    let now = Instant::now();
    let _x: Vec<_> = (1..100).map(|_x| ipsets.lookup_by_net(net0)).collect();
    println!(
        "{:.3} ms / network lookup (best case)",
        now.elapsed().as_secs_f64() / 100.0 * 1000.0
    );
}
