use std::env;

use clap::Parser;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{cursor, execute, terminal};
use futures::StreamExt;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, Stdout};
use tokio::net::unix::{ReadHalf, WriteHalf};
use tokio::net::UnixStream;

const IAC: u8 = 255;
const SB: u8 = 250;
const SE: u8 = 240;
const TELOPT_TTYPE: u8 = 24;
const TELOPT_NAWS: u8 = 31;

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

struct VppSh<'a> {
    vppctl: bool,
    stdout: Stdout,
    term_reader: EventStream,
    rd: ReadHalf<'a>,
    wr: WriteHalf<'a>,
    response: [u8; 1024],
    win_size: (u16, u16),
}

impl Drop for VppSh<'_> {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable raw mode")
    }
}

impl VppSh<'_> {
    async fn sock_wr(&mut self, buf: &[u8]) -> io::Result<()> {
        self.wr.write_all(buf).await?;
        Ok(())
    }

    async fn term_wr(&mut self, buf: &[u8]) -> io::Result<()> {
        self.stdout.write_all(buf).await?;
        self.stdout.flush().await?;
        Ok(())
    }

    async fn term_wr_response(&mut self, n: usize) -> io::Result<()> {
        self.stdout.write_all(&self.response[0..n]).await?;
        self.stdout.flush().await?;
        Ok(())
    }

    async fn win_resize(&mut self) -> io::Result<()> {
        self.win_size = terminal::size()?;
        self.sock_wr(&[
            IAC,
            SB,
            TELOPT_NAWS,
            (self.win_size.0 >> 8) as u8,
            self.win_size.0 as u8,
            (self.win_size.1 >> 8) as u8,
            self.win_size.1 as u8,
            IAC,
            SE,
        ])
        .await?;

        Ok(())
    }

    async fn ctl_handle(&mut self, event: Event) -> io::Result<()> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.vppctl = false;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
            }) => {
                self.wr.write_all(&[c as u8]).await?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.wr.write_all(b"\n").await?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.wr.write_all(b"\x10").await?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.wr.write_all(b"\x0e").await?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.wr.write_all(b"\x10").await?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.wr.write_all(b"\x0e").await?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.wr.write_all(b"\t").await?;
            }
            evt => {
                println!("{:?}\r", evt);
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    print_header();

    let term_type = env::var("TERM").expect("Could not determine terminal type");

    let args = Cli::parse();

    let stdout = io::stdout();

    terminal::enable_raw_mode().expect("Could not turn terminal on Raw mode");
    let term_reader = EventStream::new();
    let mut stream = UnixStream::connect(args.socket).await?;
    let (rd, wr) = stream.split();
    let response = [0; 1024];

    let mut vppsh = VppSh {
        stdout,
        term_reader,
        rd,
        wr,
        response,
        vppctl: false,
        win_size: terminal::size()?,
    };

    vppsh.sock_wr(&[IAC, SB, TELOPT_TTYPE, 0]).await?;
    vppsh.sock_wr(term_type.as_bytes()).await?;
    vppsh.sock_wr(&[IAC, SE]).await?;

    vppsh.win_resize().await?;

    loop {
        tokio::select! {
            Ok(n) = vppsh.rd.read(&mut vppsh.response) => {
                if n == 0 {
                    break;
                };
                if vppsh.vppctl {
                    vppsh.term_wr_response(n).await?;
                    // write!(String::from_utf8_lossy(&response[0..n]));
                    // println!("{}-{:?}", n, &response[0..n]);
                } else {

                }

            }

            event_result = vppsh.term_reader.next() =>  {
                let event = match event_result {
                    None => break,
                    Some(Err(_)) => break, // IO error on stdin
                    Some(Ok(event)) => event,
                };

                if let Event::Resize(_, _) = event {
                    vppsh.win_resize().await?;
                } else {
                    if vppsh.vppctl {
                        vppsh.ctl_handle(event).await?;
                    } else {
                        match event {
                            Event::Key(KeyEvent{code: KeyCode::Char('v'),modifiers: KeyModifiers::CONTROL }) => {
                                execute!(std::io::stdout(), terminal::Clear(terminal::ClearType::All))?;
                                execute!(std::io::stdout(), cursor::MoveTo(0, 0))?;
                                vppsh.term_wr(b"Enter vppctl interactive mode\n\rvpp# ").await?;
                                vppsh.vppctl = true;

                            }
                            Event::Key(KeyEvent{code: KeyCode::Char('q'),modifiers: KeyModifiers::CONTROL }) => {
                                break;
                            }
                            evt => {println!("vppsh: {:?}\r", evt);}
                        }
                    }
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
