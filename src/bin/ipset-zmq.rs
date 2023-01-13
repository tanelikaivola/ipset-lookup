use anyhow::Result;
use clap::{crate_authors, crate_version, App, Arg};
use ipset_lookup::lookup::LookupSets;
use std::thread;
use std::{net::Ipv4Addr, sync::Arc};

fn app_params<'a, 'b>() -> App<'a, 'b> {
    #[allow(unused_mut)]
    let mut app = App::new("ipset-zmq")
    .about("Serve ipset data over zmq")
    .version(crate_version!())
    .author(crate_authors!())
    .arg(Arg::with_name("glob")
        .long("glob")
        .short("g")
        .takes_value(true)
        .empty_values(false)
        .help("input ipset/netset files, glob syntax (defaults to: blocklist-ipsets/**/*.*set)"));
    app
}

fn worker(context: &zmq::Context, lookupsets: Arc<LookupSets>) -> ! {
    let receiver = context.socket(zmq::REP).unwrap();
    receiver
        .connect("inproc://workers")
        .expect("failed to connect worker");
    loop {
        let s = receiver
            .recv_string(0)
            .expect("worker failed receiving")
            .unwrap();

        let ip: Ipv4Addr = s.parse().expect("invalid ip");

        let feeds: Vec<_> = lookupsets.lookup_by_ip(ip);

        let out = format!(r#"{{"ip":"{ip}", "feeds":{feeds:?}}}"#);

        receiver.send(&out, 0).unwrap();
    }
}

pub fn serve(lookupsets: LookupSets) {
    let context = zmq::Context::new();
    let clients = context.socket(zmq::ROUTER).unwrap();
    let workers = context.socket(zmq::DEALER).unwrap();

    clients
        .bind("tcp://127.0.0.1:5555")
        .expect("failed to bind client router");
    workers
        .bind("inproc://workers")
        .expect("failed to bind worker dealer");

    let lookupsets_arc = Arc::new(lookupsets);
    for _ in 0..8 {
        let ctx = context.clone();
        let ls = lookupsets_arc.clone();
        thread::spawn(move || worker(&ctx, ls));
    }
    zmq::proxy(&clients, &workers).expect("failed proxying");
}

fn main() -> Result<()> {
    let app = app_params();

    let m = app.get_matches();

    let globfiles = if m.is_present("glob") {
        m.value_of("glob").unwrap()
    } else {
        "blocklist-ipsets/**/*.*set"
    };

    let ipsets = LookupSets::new(globfiles)?;

    serve(ipsets);

    Ok(())
}
