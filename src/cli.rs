use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
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

    HashObject {
        file_path: String,
    },

    Status,

    Rm {
        #[clap(long)]
        cashed: bool,

        #[clap(long)]
        forced: bool,

        #[clap(required = true)]
        paths: Vec<PathBuf>,
    },

    Add {
        #[clap(required_unless_present = "all")]
        paths: Vec<PathBuf>,
    },

    #[clap(name = "ls-files")]
    LsFiles {
        #[clap(long)]
        stage: bool,
    },

    WriteTree {
        #[clap(default_value = ".")]
        path: PathBuf,
    },

    Commit {
        #[clap(short = 'm', long)]
        message: String,

        #[clap(short = 'a', long)]
        author: Option<String>,
    },
}
