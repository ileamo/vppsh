mod conf_tui;
mod history;

use conf_tui::ConfTui;
use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    execute, terminal,
};
use gettext::Catalog;
use history::History;
use std::env;
use tokio::{
    io::{self, AsyncWriteExt, Stdout},
    net::{
        unix::{OwnedReadHalf, OwnedWriteHalf},
        UnixStream,
    },
};

#[macro_use]
extern crate tr;

const IAC: u8 = 255;
const SB: u8 = 250;
const SE: u8 = 240;
const TELOPT_TTYPE: u8 = 24;
const TELOPT_NAWS: u8 = 31;

pub enum Loop {
    Continue,
    Break,
}

pub struct VppSh<'a> {
    socket_name: &'a str,
    pub vppctl: bool,
    stdout: Stdout,
    pub term_reader: EventStream,
    pub rd: OwnedReadHalf,
    wr: OwnedWriteHalf,
    pub response: [u8; 1024],
    win_size: (u16, u16),
    ru: Catalog,
    en: Catalog,
    history: History,
    conf_tui: ConfTui,
}

impl VppSh<'_> {
    pub async fn new(socket_name: &str, en: Catalog, ru: Catalog) -> VppSh {
        let stream = UnixStream::connect(socket_name)
            .await
            .expect(&tr!("Could not connect vpp ctl socket"));
        let (rd, wr) = stream.into_split();

        VppSh {
            socket_name: socket_name,
            stdout: io::stdout(),
            term_reader: EventStream::new(),
            rd,
            wr,
            response: [0; 1024],
            vppctl: false,
            win_size: terminal::size().unwrap(),
            ru,
            en,
            history: History::new(),
            conf_tui: ConfTui::new(),
        }
    }

    pub async fn connect(&mut self) -> io::Result<()> {
        let stream = UnixStream::connect(&self.socket_name).await?;
        let (rd, wr) = stream.into_split();
        self.rd = rd;
        self.wr = wr;
        Ok(())
    }

    async fn sock_wr(&mut self, buf: &[u8]) -> io::Result<()> {
        self.wr.write_all(buf).await?;
        Ok(())
    }

    async fn term_wr(&mut self, buf: &[u8]) -> io::Result<()> {
        self.stdout.write_all(buf).await?;
        self.stdout.flush().await?;
        Ok(())
    }

    pub async fn term_wr_response(&mut self, n: usize) -> io::Result<()> {
        self.stdout.write_all(&self.response[0..n]).await?;
        self.stdout.flush().await?;

        self.history.collect_history(&self.response[0..n]);

        // println!("{:?}\r", &self.response[0..n]);

        Ok(())
    }

    pub async fn win_resize(&mut self) -> io::Result<()> {
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

    pub async fn ctl_init(&mut self) -> io::Result<()> {
        let term_type = env::var("TERM").expect("Could not determine terminal type");

        self.sock_wr(&[IAC, SB, TELOPT_TTYPE, 0]).await?;
        self.sock_wr(term_type.as_bytes()).await?;
        self.sock_wr(&[IAC, SE]).await?;
        self.win_resize().await?;

        Ok(())
    }

    pub async fn sh_handle(&mut self, event: Event) -> io::Result<Loop> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('i'),
                modifiers: KeyModifiers::NONE,
            }) => {
                clear_terminal()?;
                self.term_wr(format!("{}\n\r", tr!("Enter vppctl interactive mode")).as_bytes())
                    .await?;

                self.history.reset_curr_comand();
                self.vppctl = true;
                self.wr.write_all(b"\n").await?;
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::NONE,
            })
            | Event::Resize(_, _) => {
                self.conf_tui.draw(&self.history.history);
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char('x'),
                modifiers: KeyModifiers::NONE,
            }) => {
                clear_terminal()?;
                print_header();
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::NONE,
            }) => {
                set_translator!(self.en.clone());
                clear_terminal()?;
                print_header();
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::NONE,
            }) => {
                set_translator!(self.ru.clone());
                clear_terminal()?;
                print_header();
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
            }) => return Ok(Loop::Break),
            _evt => {
                // println!("vppsh: {:?}\r", evt);
            }
        }

        Ok(Loop::Continue)
    }

    pub async fn quit_vppctl(&mut self) -> io::Result<()> {
        self.vppctl = false;
        clear_terminal()?;
        print_header();
        Ok(())
    }

    pub async fn ctl_handle(&mut self, event: Event) -> io::Result<()> {
        match event {
            Event::Resize(_, _) => {
                self.win_resize().await?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.quit_vppctl().await?;
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
                self.history.was_enter();
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
                self.wr.write_all(b"\x02").await?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.wr.write_all(b"\x06").await?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.wr.write_all(b"\x08").await?;
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

pub fn print_header() {
    let header = "                            .__\r
 ___  ________ ______  _____|  |__\r
 \\  \\/ /\\____ \\\\____ \\/  ___/  |  \\\r
  \\   / |  |_> >  |_> >___ \\|   Y  \\\r
   \\_/  |   __/|   __/____  >___|  /\r
        |__|   |__|       \\/     \\/\r
";

    println!("{}\r", header);
    println!("{}\r\n", tr!("Wrapper around vppctl"));
    println!("{}\r\n", tr!("Commands"));
    println!("i - {}\r", tr!("Enter vppctl mode"));
    println!("q - {}\r", tr!("Quit"));
    println!("e - {}\r", tr!("Set english locale"));
    println!("r - {}\r", tr!("Set russian locale"));
    println!("\n{}\r", tr!("More comands under constuction"));
}

fn clear_terminal() -> io::Result<()> {
    execute!(std::io::stdout(), terminal::Clear(terminal::ClearType::All))?;
    execute!(std::io::stdout(), cursor::MoveTo(0, 0))?;
    Ok(())
}
