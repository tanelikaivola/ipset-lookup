#![allow(unused)]

use glob::{glob, Paths};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use ipnetwork::Ipv4Network;
use std::iter::FromIterator;
use rayon::prelude::*;

fn files() -> Paths {
    glob("blocklist-ipsets-full/**/*.*set").unwrap()
}

fn glob_vec(pattern: &str) -> Vec<PathBuf> {
    glob(pattern).unwrap().map(|r| r.unwrap()).collect()
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
    ipsets.par_iter().filter(
        |(_, nets)| nets.iter().any(|net| net.contains(ip))
    ).map(
        |(name, _)| name
    ).collect()
}

fn main_sequential() {
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

fn main() {
    let files = glob_vec("blocklist-ipsets-full/**/*.*set");

    let ipsetiter : Vec<_> = files.par_iter().map(
        |path| parse_file(path.to_path_buf())
    ).collect();
    let ipsets = HashMap::<String,Vec<Ipv4Network>>::from_iter(ipsetiter);

    //    println!("{:?}", ipsets);
    println!("Ding: {:?}", lookup(&ipsets, "62.73.8.0".parse().unwrap()));
    println!("Ding: {:?}", lookup(&ipsets, "62.73.8.0".parse().unwrap()));
    println!("Ding: {:?}", lookup(&ipsets, "50.7.78.88".parse().unwrap()));
    println!("Ding: {:?}", lookup(&ipsets, "64.135.235.144".parse().unwrap()));
}