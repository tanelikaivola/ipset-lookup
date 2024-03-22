use ipset_lookup::*;

#[test]
fn test_loading() {
    let ipsets = LookupSets::new("blocklist-ipsets/**/*.*set").unwrap();
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
