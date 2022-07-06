use clap::Parser;
use crossterm::event::{Event, EventStream};
use crossterm::terminal;
use futures::StreamExt;
use tokio::io::{self, AsyncReadExt};
use tokio::net::UnixStream;

#[derive(Parser, Default)]
#[clap(version, about = "VPP shell")]
struct Cli {
    /// VPP command
    #[clap(forbid_empty_values = true, validator = validate_vpp_command)]
    command: Option<String>,

    /// VPP &cli socket path
    #[clap(default_value = "/run/vpp/cli.sock", short, long)]
    socket: String,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    vppsh::print_header();

    let args = Cli::parse();

    let stdout = io::stdout();

    terminal::enable_raw_mode().expect("Could not turn terminal on Raw mode");
    let term_reader = EventStream::new();

    let stream = UnixStream::connect(&args.socket).await?;
    let (rd, wr) = stream.into_split();

    let mut vppsh = vppsh::VppSh {
        socket_name: &args.socket,
        stdout,
        term_reader,
        rd,
        wr,
        response: [0; 1024],
        vppctl: false,
        win_size: terminal::size()?,
    };

    vppsh.ctl_init().await?;

    loop {
        tokio::select! {
            Ok(n) = vppsh.rd.read(&mut vppsh.response) => {
                if n == 0 {
                    vppsh.quit_vppctl().await?;
                    vppsh.connect().await?;
                    vppsh.ctl_init().await?;
                } else if vppsh.vppctl {
                    vppsh.term_wr_response(n).await?;
                } else {

                }
            }

            event_result = vppsh.term_reader.next() =>  {
                let event = match event_result {
                    None => break,
                    Some(Err(_)) => break,
                    Some(Ok(event)) => event,
                };

                match event {
                    Event::Resize(_, _) => {
                        vppsh.win_resize().await?;
                    }
                    event => {
                        if vppsh.vppctl {
                            vppsh.ctl_handle(event).await?;
                        } else {
                            if let vppsh::Loop::Break = vppsh.sh_handle(event).await? {
                                break;
                            }

                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_vpp_command(name: &str) -> Result<(), String> {
    if name.trim().len() == 0 {
        Err(String::from("command cannot be empty"))
    } else {
        Ok(())
    }
}
