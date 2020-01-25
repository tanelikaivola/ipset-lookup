use glob::{glob, Paths};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use ipnetwork::Ipv4Network;
use std::iter::FromIterator;

fn files() -> Paths {
    glob("blocklist-ipsets/**/*.*set").unwrap()
}

fn parse_file(path : std::path::PathBuf) -> (String, Vec<Ipv4Network>) {
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

fn lookup(ipsets: &HashMap<String,Vec<Ipv4Network>>, ip:Ipv4Addr) -> HashSet<&String> {
    ipsets.iter().filter(
        |(_, nets)| nets.iter().any(|net| net.contains(ip))
    ).map(
        |(name, _)| name
    ).collect()
}

fn main() {
    let ipsetiter = files().map(
        |p| p.unwrap()
    ).map(
        |path| parse_file(path)
    );
    let ipsets = HashMap::<String,Vec<Ipv4Network>>::from_iter(ipsetiter);

    //    println!("{:?}", ipsets);
    println!("Ding: {:?}", lookup(&ipsets, "62.73.8.0".parse().unwrap()));
    println!("Ding: {:?}", lookup(&ipsets, "62.73.8.0".parse().unwrap()));
    println!("Ding: {:?}", lookup(&ipsets, "50.7.78.88".parse().unwrap()));
    println!("Ding: {:?}", lookup(&ipsets, "14.192.4.35".parse().unwrap()));
}
