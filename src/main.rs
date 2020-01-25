use glob::{glob, Paths};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use ipnetwork::Ipv4Network;

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

fn lookup(ipsets: HashMap<String,Vec<Ipv4Network>>, ip:Ipv4Addr) -> HashSet<String> {
    let mut out : HashSet<String>  = HashSet::new();

    for (name, nets) in ipsets {
        for net in nets {
            if net.contains(ip) {
                out.insert(name.clone());
            }
        }
    }

    out
}

fn main() {
    let mut ipsets : HashMap<String,Vec<Ipv4Network>> = HashMap::new();

    for path in files() {
        let path = path.unwrap();
        let (name, ipset) = parse_file(path);
        ipsets.insert(name, ipset);
    }

//    println!("{:?}", ipsets);
    println!("Ding: {:?}", lookup(ipsets, "62.73.8.0".parse().unwrap()));
}
