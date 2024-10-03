use std::fs;
use std::io::Read;
use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use flate2::read::ZlibDecoder;
use lazy_static::lazy_static;


#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        #[clap(short = 't')]
        show_type: bool,

        #[clap(short = 's')]
        show_size: bool,

        object_hash: String,
    },
    HelpCommand,
}

lazy_static! {
    static ref VCS_DIR: String = ".vcs".to_string();
    static ref OBJ_DIR: String = format!("{}/objects", *VCS_DIR);
    static ref REFS_DIR: String = format!("{}/refs", *VCS_DIR);
    static ref HEAD_DIR: String = format!("{}/HEAD", *VCS_DIR);
}

fn init_command() -> Result<()> {
    fs::create_dir(&*VCS_DIR).context("Failed to create .vcs directory")?;
    fs::create_dir(&*OBJ_DIR).context("Failed to create .vcs/objects directory")?;
    fs::create_dir(&*REFS_DIR).context("Failed to create .vcs/refs directory")?;
    fs::write(&*HEAD_DIR, "ref: refs/heads/main\n").context("Failed to write to .vcs/HEAD file")?;

    println!("Initialized vcs directory"); 
    Ok(())
}

fn cat_file_command(pretty_print: bool, object_hash: String, show_type: bool, show_size: bool) -> Result<()> {
    let object_path = format!(".vcs/objects/{}/{}", &object_hash[0..2], &object_hash[2..]);
    let file = std::fs::File::open(&object_path).with_context(|| format!("Failed to open object file: {}", object_hash))?;

    let mut decoder = ZlibDecoder::new(file);
    let mut decoder_data = Vec::new();
    decoder.read_to_end(&mut decoder_data).context("Failed to read object data")?;

    // Split the header and the data
    let header_end = decoder_data.iter().position(|&b| b == b'\0').context("Failed to find header end")?;
    let header = String::from_utf8_lossy(&decoder_data[..header_end]);

    let data = &decoder_data[header_end + 1..];
    let _compressed_data = std::fs::metadata(&object_path).context("Failed to get the file metadata")?.len();

    match (show_type, show_size, pretty_print) {
        (true, false, false) => {
            let object_type = header.split(' ').next().unwrap_or("unknown");
            println!("{}", object_type);
        },
        (false, true, false) => {
            println!("{}", data.len());
        },
        (false, false, true) | (false, false, false) => {
            print!("{}", String::from_utf8_lossy(data));
        },
        _ => {
            let object_type = header.split(' ').next().unwrap_or("unknown");
            println!("{}", object_type);
            println!("{}", data.len());
            print!("{}", String::from_utf8_lossy(data));
        }
    }

    Ok(())
}

fn help_command() -> Result<()> {
    println!("Usage: vcs [COMMAND]");
    println!("Commands:");
    println!("  init        Initialize a new vcs repository");
    println!("  cat-file -p <hash>  Display object content with header");
    println!("  help        Display this help message");

    Ok(())
}

fn read_object() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Init => init_command()?,
        Commands::CatFile { pretty_print, object_hash, show_type, show_size } => cat_file_command(pretty_print, object_hash, show_type, show_size)?,
        Commands::HelpCommand => help_command()?,
    }

    Ok(())
}

fn main() {
    if let Err(e) = read_object() {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}
