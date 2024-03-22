#![allow(dead_code)]

use glob::glob;
use ipnetwork::{IpNetworkError, Ipv4Network};
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::{fmt, net::AddrParseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("glob pattern error")]
    GlobPattern(#[from] glob::PatternError),
    #[error("glob error")]
    GlobError(#[from] glob::GlobError),
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("IP Network parse error")]
    IpNetworkError(#[from] IpNetworkError),
    #[error("Parsing IP address failed")]
    AddrParseError(#[from] AddrParseError),
}

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
    let file = File::open(path)?;
    let buffered = BufReader::new(file);
    let lines = buffered.lines();
    let comments: Vec<_> = lines
        .map(Result::unwrap)
        .take_while(|l| l.starts_with('#'))
        .filter(|l| l.starts_with("# Category        : "))
        .map(|l| l.replace("# Category        : ", ""))
        .collect();
    let category: String = if comments.len() == 1 {
        String::from(&comments[0])
    } else {
        String::from("other")
    };

    // reinitialize the reader (FIXME)
    let file = File::open(path)?;
    let buffered = BufReader::new(file);
    let lines = buffered
        .lines()
        .map(Result::unwrap)
        .filter(|l| !l.starts_with('#'));
    let (nets, ips): (Vec<_>, Vec<_>) = lines.partition(|l| l.contains('/'));

    let nets = nets
        .iter()
        .map(|l| l.parse())
        .collect::<Result<Vec<Ipv4Network>, IpNetworkError>>()?;

    let ips: Vec<Ipv4Addr> = ips
        .iter()
        .map(|l| l.parse())
        .collect::<Result<Vec<Ipv4Addr>, AddrParseError>>()?;

    let name = path
        .file_stem()
        .expect("can't extract file stem")
        .to_os_string()
        .into_string()
        .expect("can't convert OsString to String");

    Ok(NetSet {
        feed: NetSetFeed { category, name },
        nets,
        ips,
    })
}

#[derive(Clone)]
pub struct LookupSets {
    data: Vec<NetSet>,
}

impl LookupSets {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        //        let files = glob_vec(pattern)?;
        let files: Vec<PathBuf> = glob(pattern)?.collect::<Result<Vec<PathBuf>, _>>()?;
        let parsed: Result<Vec<NetSet>, Error> = files.par_iter().map(parse_file).collect();

        Ok(Self { data: parsed? })
    }

    #[must_use]
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

    #[must_use]
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
