#![allow(dead_code)]

use glob::glob;
use ipnetwork::Ipv4Network;
use rayon::prelude::*;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use std::path::PathBuf;

pub trait Lookup {
    fn lookup_by_ip(&self, ip: Ipv4Addr) -> bool;
}

impl Lookup for Vec<Ipv4Network> {
    fn lookup_by_ip(&self, ip: Ipv4Addr) -> bool {
        self.iter().any(|net| net.contains(ip))
    }
}

impl Lookup for Vec<Ipv4Addr> {
    fn lookup_by_ip(&self, ip: Ipv4Addr) -> bool {
        self.iter().any(|other| *other == ip)
    }
}

fn glob_vec(pattern: &str) -> Vec<PathBuf> {
    glob(pattern).unwrap().map(|r| r.unwrap()).collect()
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct NetSetFeed {
    category: String,
    name: String,
}
impl fmt::Debug for NetSetFeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#""{}/{}""#, self.category, self.name)
    }
}

struct NetSet {
    feed: NetSetFeed,
    nets: Vec<Ipv4Network>,
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

fn parse_file(path: &std::path::PathBuf) -> NetSet {
    let name = path.file_stem().unwrap().to_str().unwrap().to_string();

    let file = File::open(path).unwrap();
    let buffered = BufReader::new(file);
    let mut lines = buffered.lines();
    let comments: Vec<_> = lines
        .by_ref()
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
    let file = File::open(path).unwrap();
    let buffered = BufReader::new(file);
    let mut lines = buffered.lines();
    let nets: Vec<Ipv4Network> = lines
        .by_ref()
        .map(|l| l.unwrap())
        .filter(|l| !l.starts_with('#'))
        .map(|l| l.parse().unwrap())
        .collect();

    NetSet {
        feed: NetSetFeed { name, category },
        nets,
    }
}

pub struct LookupSets {
    data: Vec<NetSet>,
}

impl LookupSets {
    pub fn new(glob: &str) -> LookupSets {
        let files = glob_vec(glob);

        let ipsetiter: Vec<_> = files
            .par_iter()
            .map(|path| parse_file(&path.to_path_buf()))
            .collect();
        LookupSets { data: ipsetiter }
    }
    pub fn lookup_by_ip(&self, ip: Ipv4Addr) -> Vec<&NetSetFeed> {
        let mut output: Vec<_> = self
            .data
            .par_iter()
            .filter(|netset| netset.nets.lookup_by_ip(ip))
            .map(|netset| &netset.feed)
            .collect();
        output.sort();
        output
    }
    pub fn lookup_by_net(&self, other: Ipv4Network) -> Vec<&NetSetFeed> {
        let mut output: Vec<_> = self
            .data
            .par_iter()
            .filter(|netset| netset.nets.iter().any(|net| net.overlaps(other)))
            .map(|netset| &netset.feed)
            .collect();
        output.sort();
        output
    }
}

#[test]
fn test_sanity() {
    let ipsets = LookupSets::new("blocklist-ipsets/**/*.*set");
    let ip: Ipv4Addr = "8.8.8.8".parse().expect("Invalid IP");
    ipsets.lookup_by_ip(ip);

    let mut rawips: Vec<Ipv4Addr> = Vec::new();
    rawips.insert(0, "8.8.8.8".parse().unwrap());
    let ip: Ipv4Addr = "8.8.8.8".parse().unwrap();
    assert!(rawips.lookup_by_ip(ip), "lookup_by_ip is not eq");
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
