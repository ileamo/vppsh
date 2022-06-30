use clap::Parser;

#[derive(Parser, Default)]
#[clap(version, about = "VPP shell")]
struct Cli {
    /// VPP command
    #[clap(forbid_empty_values = true, validator = validate_vpp_command)]
    command: Option<String>,

    /// VPP cli socket path
    #[clap(default_value = "/run/vpp/cli.sock", parse(from_os_str), short, long)]
    socket: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Some(cmd) => exec_vpp_command(args.socket, cmd),
        None => interactive_mode(args.socket),
    };
}

fn validate_vpp_command(name: &str) -> Result<(), String> {
    if name.trim().len() == 0 {
        Err(String::from("command cannot be empty"))
    } else {
        Ok(())
    }
}

fn exec_vpp_command(socket_name: std::path::PathBuf, cmd: String) {
    println!("Connect to socket {}", socket_name.display());
    println!("vppsh# {}", cmd);
}

fn interactive_mode(socket_name: std::path::PathBuf) {
    print_header();
    println!("Connect to socket {}", socket_name.display());
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
