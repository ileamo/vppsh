use std::env;

use clap::Parser;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use futures::StreamExt;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
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

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable raw mode")
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    print_header();

    let term_type = env::var("TERM").expect("Could not determine terminal type");

    let args = Cli::parse();

    let mut stdout = io::stdout();

    let _clean_up = CleanUp;
    terminal::enable_raw_mode().expect("Could not turn terminal on Raw mode");
    let mut reader = EventStream::new();

    let mut stream = UnixStream::connect(args.socket).await?;
    let (mut rd, mut wr) = stream.split();
    let mut response = [0; 1024];

    wr.write_all(b"\xff\xfa\x18\x00").await?;
    wr.write_all(term_type.as_bytes()).await?;
    wr.write_all(b"\xff\xf0").await?;

    loop {
        tokio::select! {
            Ok(n) = rd.read(&mut response) => {
                if n == 0 {
                    break;
                };
                stdout.write_all(&response[0..n]).await?;
                stdout.flush().await?;
                // write!(String::from_utf8_lossy(&response[0..n]));
                // println!("{}-{:?}", n, &response[0..n]);

            }

            event_result = reader.next() =>  {
                let event = match event_result {
                    None => break,
                    Some(Err(_)) => break, // IO error on stdin
                    Some(Ok(event)) => event,
                };
                match event {
                    Event::Key(KeyEvent{code: KeyCode::Char('q'),modifiers: KeyModifiers::CONTROL }) => {
                        break;
                    }
                    Event::Key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::NONE }) => {
                        wr.write_all(&[c as u8]).await?;
                    }
                    Event::Key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE }) => {
                        wr.write_all(b"\n").await?;
                    }
                    Event::Key(KeyEvent { code: KeyCode::Up, modifiers: KeyModifiers::NONE }) => {
                        wr.write_all(b"\x10").await?;
                    }
                    Event::Key(KeyEvent { code: KeyCode::Down, modifiers: KeyModifiers::NONE }) => {
                        wr.write_all(b"\x0e").await?;
                    }
                    Event::Key(KeyEvent { code: KeyCode::Tab, modifiers: KeyModifiers::NONE }) => {
                        wr.write_all(b"\t").await?;
                    }
                    evt => {println!("{:?}\r", evt);}
                }
            }
        };
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
