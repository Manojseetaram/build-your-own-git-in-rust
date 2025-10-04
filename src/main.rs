use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

pub(crate) mod commands;
pub(crate) mod objects;

/// Simple program to gree1t a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

/// Doc comment
#[derive(Debug, Subcommand)]
enum Command {
    /// Doc comment
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    },
    LsTree {
        #[clap(long)]
        name_only: bool,

        tree_hash: String,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    match args.command {
        Command::Init => {
            fs::create_dir_all(".git/objects")?;
            fs::create_dir_all(".git/refs")?;
            fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
            println!("Initialized git directory")
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            commands::cat_file::invoke(pretty_print, &object_hash)?;
        }
        Command::HashObject { write, file } => {
            let file_str = file
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in file path"))?;
            commands::hash_object::invoke(write, file_str)?;
        }
        Command::LsTree {
            name_only,
            tree_hash,
        } => {
            commands::ls_tree::invoke(name_only, &tree_hash)?;
        }
    }

    Ok(())
}
