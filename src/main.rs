use glob::glob;
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use ipnetwork::Ipv4Network;
use std::iter::FromIterator;
use rayon::prelude::*;

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

struct IPSets {
    data : HashMap<String,Vec<Ipv4Network>>
}

impl IPSets {
    fn new() -> IPSets {
        let files = glob_vec("blocklist-ipsets-full/**/*.*set");

        let ipsetiter : Vec<_> = files.par_iter().map(
            |path| parse_file(&path.to_path_buf())
        ).collect();
        IPSets {data: HashMap::from_iter(ipsetiter)}
    }
    fn lookup_by_ip(&self, ip:Ipv4Addr) -> Vec<&String> {
        let mut output = self.data.par_iter().filter(
            |(_, nets)| nets.iter().any(|net| net.contains(ip))
        ).map(
            |(name, _)| name
        ).collect::<Vec<_>>();
        output.sort();
        output
    }
    fn lookup_by_str(&self, ip:&str) -> Vec<&String> {
        let ip : Ipv4Addr = ip.parse().unwrap();
        self.lookup_by_ip(ip)
    }
    fn lookup_by_net(&self, ip:Ipv4Network) -> Vec<&String> {
        let mut output = self.data.par_iter().filter(
            |(_, nets)| nets.iter().any(|net| net.overlaps(ip))
        ).map(
            |(name, _)| name
        ).collect::<Vec<_>>();
        output.sort();
        output
    }
}

#[test]
fn test_speed() {
    use std::time::Instant;
    let now = Instant::now();

    let ipsets = IPSets::new();
    let now = Instant::now();
    let _x : Vec<_> = (1..100).map(|_x|
        ipsets.lookup_by_ip("64.135.235.144".parse().expect("Invalid IP"))
    ).collect();
    println!("{} ms / ip lookup", now.elapsed().as_secs_f64()/100.0*1000.0);

    let now = Instant::now();
    let _x : Vec<_> = (1..100).map(|_x|
        ipsets.lookup_by_net("64.135.235.144".parse().expect("Invalid network"))
    ).collect();
    println!("{} ms / network lookup (maybe worst case)", now.elapsed().as_secs_f64()/100.0*1000.0);

    let now = Instant::now();
    let _x : Vec<_> = (1..100).map(|_x|
        ipsets.lookup_by_net("0.0.0.0/0".parse().expect("Invalid network"))
    ).collect();
    println!("{} ms / network lookup (best case)", now.elapsed().as_secs_f64()/100.0*1000.0);    
}

fn main() {

}