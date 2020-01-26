use glob::glob;
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use ipnetwork::Ipv4Network;
use std::iter::FromIterator;
use rayon::prelude::*;

pub trait Lookup {
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

pub struct LookupSets {
    data : HashMap<String,Vec<Ipv4Network>>
}

impl LookupSets {
    pub fn new(glob : &str) -> LookupSets {
        let files = glob_vec(glob);

        let ipsetiter : Vec<_> = files.par_iter().map(
            |path| parse_file(&path.to_path_buf())
        ).collect();
        LookupSets {data: HashMap::from_iter(ipsetiter)}
    }
    pub fn lookup_by_ip(&self, ip:&Ipv4Addr) -> Vec<&String> {
        let mut output = self.data.par_iter().filter(
            |(_, nets)| nets.lookup_by_ip(ip)
        ).map(
            |(name, _)| name
        ).collect::<Vec<_>>();
        output.sort();
        output
    }
    pub fn lookup_by_str(&self, ip:&str) -> Vec<&String> {
        let ip : Ipv4Addr = ip.parse().expect("invalid ip address");
        self.lookup_by_ip(&ip)
    }
    pub fn lookup_by_net(&self, other:&Ipv4Network) -> Vec<&String> {
        let mut output = self.data.par_iter().filter(
            |(_, nets)| nets.iter().any(|net| net.overlaps(*other))
        ).map(
            |(name, _)| name
        ).collect::<Vec<_>>();
        output.sort();
        output
    }
}