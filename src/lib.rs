#![forbid(unsafe_code)]

pub mod lookup;
pub use ipnetwork::Ipv4Network;
pub use lookup::{Lookup, LookupSets, NetSetFeed};
pub use std::net::Ipv4Addr;
