use anyhow::{Error, Result};
use std::time::Instant;

#[cfg(feature = "zmq")]
fn main() -> Result<()> {
    let context = zmq::Context::new();
    let t0 = Instant::now();
    let req = context.socket(zmq::REQ)?;
    req.connect("tcp://127.0.0.1:5555")?;
    req.send("8.8.8.8", 0)?;
    let recv_info = req.recv_string(0)?;
    let ip_info = recv_info.map_err(|_| Error::msg("Failed to receive data"))?;
    println!("{ip_info}");
    let elapsed = t0.elapsed().as_secs_f64();
    println!("Elapsed: {elapsed} s");
    Ok(())
}

#[cfg(not(feature = "zmq"))]
fn main() {
    compile_error!("Enable zmq feature to compile this example")
}
