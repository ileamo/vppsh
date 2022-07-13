use std::str::FromStr;

use clap::Parser;
use crossterm::terminal;
use futures::StreamExt;
use gettext::Catalog;
use rust_embed::RustEmbed;
use sys_locale::get_locale;
use tokio::io::{self, AsyncReadExt};

#[macro_use]
extern crate tr;

struct CleanUp;
impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable raw mode")
    }
}

#[derive(Default)]
enum Locale {
    #[default]
    Sys,
    Ru,
    En,
}

impl FromStr for Locale {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Locale::En),
            "ru" => Ok(Locale::Ru),
            "sys" => Ok(Locale::Sys),
            _ => Err("value must be 'en', 'ru' or 'sys'".to_string()),
        }
    }
}

#[derive(Parser, Default)]
#[clap(version, about = "VPP shell")]
struct Cli {
    /// VPP cli socket path
    #[clap(default_value = "/run/vpp/cli.sock", short, long)]
    socket: String,

    /// Set locale
    #[clap(default_value = "sys", short, long)]
    locale: Locale,
}

#[derive(RustEmbed)]
#[folder = "i18n/mo"]
struct Asset;

#[tokio::main]
async fn main() -> io::Result<()> {
    let _clean_up = CleanUp;

    let ru_mo = Asset::get("ru/vppsh.mo").expect("could not find ru/vppsh.mo");
    let ru_mo = ru_mo.data.as_ref();
    let ru = Catalog::parse(ru_mo).expect("could not parse the catalog ru/vppsh.mo");

    let en_mo = Asset::get("en/vppsh.mo").expect("could not find en/vppsh.mo");
    let en_mo = en_mo.data.as_ref();
    let en = Catalog::parse(en_mo).expect("could not parse the catalog en/vppsh.mo");

    let args = Cli::parse();

    set_translator!(match args.locale {
        Locale::Ru => ru.clone(),
        Locale::En => en.clone(),
        Locale::Sys => {
            match get_locale() {
                Some(lcl) if &lcl[0..2] == "ru" => ru.clone(),
                _ => en.clone(),
            }
        }
    });

    terminal::enable_raw_mode().expect("Could not turn terminal on Raw mode");

    let mut vppsh = vppsh::VppSh::new(&args.socket, en, ru).await;

    vppsh.ctl_init().await?;

    vppsh::print_header();

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
    Ok(())
}
