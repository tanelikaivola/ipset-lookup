use anyhow::Result;
use clap::{ArgGroup, Parser, Subcommand};
use ipnetwork::Ipv4Network;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use std::result::Result as StdResult;
use thiserror::Error;

#[cfg(feature = "update")]
#[derive(Error, Debug)]
enum GitError {
    #[error("failed to clone repository")]
    Repository(#[from] git2::Error),
    #[error("io error")]
    Io(#[from] std::io::Error),
}

use ipset_lookup::lookup::LookupSets;

/// Fast lookup through ipset data
#[derive(Parser)]
#[command(name = "ipset-lookup", version, author)]
struct Cli {
    /// Input ipset/netset files, glob syntax
    #[arg(
        long,
        short = 'g',
        global = true,
        default_value = "blocklist-ipsets/**/*.*set"
    )]
    glob: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a lookup
    #[command(group(
        ArgGroup::new("find").multiple(true).required(true),
    ))]
    Lookup {
        /// Find IPs in the file from blocklists
        #[arg(long, short = 'f', group = "find", required = false)]
        file: Option<Vec<String>>,

        /// Find an IP on any of the blocklists
        #[arg(long, short = 'i', group = "find", required = false)]
        ip: Option<Vec<String>>,

        /// Find any IP in the network on any of the blocklists
        #[arg(long, short = 'n', group = "find", required = false)]
        net: Option<Vec<String>>,
    },

    #[cfg(feature = "bench")]
    /// Run a quick benchmark
    Bench,

    #[cfg(feature = "update")]
    /// Update blocklist-ipsets in current directory
    Update,
}

#[cfg(feature = "update")]
fn update_command() -> StdResult<(), GitError> {
    use git2::{Repository, ResetType};

    let url = "https://github.com/firehol/blocklist-ipsets.git";
    let path = r"blocklist-ipsets";
    let repo = match Repository::open(path) {
        Ok(repo) => repo,
        Err(e1) => match Repository::clone(url, path) {
            Ok(repo) => repo,
            Err(e2) => panic!("failed to clone: {} and {}", e1, e2),
        },
    };
    repo.reset(&repo.revparse_single("HEAD")?, ResetType::Hard, None)?;

    repo.find_remote("origin")?.fetch(&["master"], None, None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
    let analysis = repo.merge_analysis(&[&fetch_commit])?;
    if analysis.0.is_up_to_date() {
        Ok(())
    } else {
        let refname = format!("refs/heads/{}", "master");
        let mut reference = repo.find_reference(&refname)?;
        reference.set_target(fetch_commit.id(), "Fast-Forward")?;
        repo.set_head(&refname)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(GitError::Repository)
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let globfiles = &cli.glob;

    match &cli.command {
        Commands::Lookup { file, ip, net } => {
            let ipsets = LookupSets::new(globfiles)?;

            if let Some(files) = file {
                for path in files {
                    let file = File::open(path)?;
                    let buffered = BufReader::new(file);
                    let data = buffered
                        .lines()
                        .map(|l| l.expect("could not read line"))
                        .filter(|l| !l.starts_with('#'))
                        .map(|l| l.parse().expect("invalid ip"));
                    for ip in data {
                        let result = ipsets.lookup_by_ip(ip);
                        println!(r#"{{"ip":"{ip}", "feeds":{result:?}}}"#);
                    }
                }
            }
            if let Some(ips) = ip {
                let ips: Vec<Ipv4Addr> = ips
                    .iter()
                    .map(|ip| ip.parse().expect("invalid ip address"))
                    .collect();
                for ip in ips {
                    let result = ipsets.lookup_by_ip(ip);
                    println!(r#"{{"ip":"{ip}", "feeds":{result:?}}}"#);
                }
            }
            if let Some(nets) = net {
                let nets: Vec<Ipv4Network> = nets
                    .iter()
                    .map(|ip| ip.parse().expect("invalid net"))
                    .collect();
                for net in nets {
                    let result = ipsets.lookup_by_net(net);
                    println!(r#"{{"ip":"{net}", "feeds":{result:?}}}"#);
                }
            }
        }
        #[cfg(feature = "bench")]
        Commands::Bench => test_speed(globfiles),
        #[cfg(feature = "update")]
        Commands::Update => update_command()?,
    }
    Ok(())
}

#[cfg(feature = "bench")]
pub fn test_speed(glob: &str) {
    use std::time::Instant;
    let now = Instant::now();
    let ipsets = LookupSets::new(glob).unwrap();
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
