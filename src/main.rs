use clap::Parser;
use std::io::prelude::*;
use std::time::Duration;
use std::{
    io::Write,
    os::unix::net::{UnixListener, UnixStream},
};

#[derive(Parser, Default)]
#[clap(version, about = "VPP shell")]
struct Cli {
    /// VPP command
    #[clap(forbid_empty_values = true, validator = validate_vpp_command)]
    command: Option<String>,

    /// VPP cli socket path
    #[clap(default_value = "/run/vpp/cli.sock", short, long)]
    socket: String,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Some(cmd) => exec_vpp_command(&args.socket, cmd),
        None => interactive_mode(&args.socket),
    };
}

fn validate_vpp_command(name: &str) -> Result<(), String> {
    if name.trim().len() == 0 {
        Err(String::from("command cannot be empty"))
    } else {
        Ok(())
    }
}

fn exec_vpp_command(socket_name: &str, cmd: String) {
    let mut stream = UnixStream::connect(socket_name).unwrap();
    stream
        .set_read_timeout(Some(Duration::new(1, 0)))
        .expect("Couldn't set read timeout");

    let mut response = [0; 256];
    loop {
        let res = stream.read(&mut response);
        if let Ok(n) = res {
            println!("{} - {:?}", n, &response[0..n])
        } else {
            break
        }
    }
    stream.write_all(cmd.as_bytes()).unwrap();
    stream.write_all(b"\n").unwrap();
    loop {
        let res = stream.read(&mut response);
        if let Ok(n) = res {
            println!("{} - {:?}", String::from_utf8_lossy(&response[0..n]), &response[0..n])
        } else {
            break
        }
    }
}

fn interactive_mode(socket_name: &str) {
    print_header();
    println!("Connect to socket {}", socket_name);
    println!("vppsh# interactive mode not yet implemented 😕");
}

fn print_header() {
    let header = r#"                            .__
 ___  ________ ______  _____|  |__
 \  \/ /\____ \\____ \/  ___/  |  \
  \   / |  |_> >  |_> >___ \|   Y  \
   \_/  |   __/|   __/____  >___|  /
        |__|   |__|       \/     \/
"#;

    println!("{}", header);
}
