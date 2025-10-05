use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

pub(crate) mod commands;
pub(crate) mod objects;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
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
    WriteTree,
    CommitTree {
        #[clap(short = 'm')]
        message: String,
        #[clap(short = 'p')]
        parent_hash: Option<String>,
        tree_hash: String,
    },
    Clone {
        url: String,
        dir: String,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    eprintln!("Logs from your program will appear here!");

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
        Command::WriteTree => {
            commands::write_tree::invoke()?;
        }
        Command::CommitTree {
            message,
            tree_hash,
            parent_hash,
        } => commands::commit_tree::invoke(message, tree_hash, parent_hash)?,
        Command::Clone { url, dir } => {
            let repo = git2::Repository::clone(&url, &dir)
                .map_err(|e| anyhow::anyhow!("Failed to clone {}: {}", url, e))?;
            println!("Cloned {} to {}", url, repo.path().display());
        }
    }

    Ok(())
}
