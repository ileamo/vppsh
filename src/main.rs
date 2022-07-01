use clap::Parser;
use tokio::io::{AsyncReadExt, AsyncWriteExt, self};
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
async fn main() -> io::Result<()> {
    let args = Cli::parse();

    let stream = UnixStream::connect(args.socket).await?;
    let (mut rd, mut wr) = stream.into_split();

    let rd_handle = tokio::spawn(async move {
        let mut response = [0; 256];
        loop {
            let n = rd.read(&mut response).await?;
            if n == 0 {
                break;
            };
            println!("\n{}", String::from_utf8_lossy(&response[0..n]));
            // println!("{}-{:?}", n, &response[0..n]);
        }
        Ok::<_, io::Error>(())
    });

    let wr_handle = tokio::spawn(async move {
        if let Some(cmd) = args.command {
            let ttype_command = b"\xff\xfa\x18\x00vppctl\xff\xf0";
            wr.write_all(ttype_command).await?;
            wr.write_all(cmd.as_bytes()).await?;
            return Ok::<_, io::Error>(());
        }
        print_header();
        println!("vppsh# interactive mode not yet implemented 😕");
        Ok::<_, io::Error>(())
    });

    wr_handle.await??;
    rd_handle.await??;
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
