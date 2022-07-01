use clap::Parser;
use std::io::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

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

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Cli::parse();

    let stream = UnixStream::connect(args.socket).await?;
    let (mut read, mut write) = stream.into_split();

    let reader_handle = tokio::spawn(async move {
        let mut response = [0; 256];
        loop {
            let n = read.read(&mut response).await.unwrap();
            if n == 0 {
                break;
            };
            println!("\n{}", String::from_utf8_lossy(&response[0..n]));
            // println!("{}-{:?}", n, &response[0..n]);
        }
    });

    let writer_handle = tokio::spawn(async move {
        if let Some(cmd) = args.command {
            let ttype_command = b"\xff\xfa\x18\x00vppctl\xff\xf0";
            write.write_all(ttype_command).await.unwrap();
            write.write_all(cmd.as_bytes()).await.unwrap();
            return;
        }
        print_header();
        println!("vppsh# interactive mode not yet implemented 😕");
    });

    writer_handle.await?;
    reader_handle.await?;
    Ok(())
}

fn validate_vpp_command(name: &str) -> Result<(), String> {
    if name.trim().len() == 0 {
        Err(String::from("command cannot be empty"))
    } else {
        Ok(())
    }
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
