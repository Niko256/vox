use crate::commands::commit::get_current_commit;
use crate::objects::commit_object::Commit;
use crate::utils::OBJ_DIR;
use anyhow::Result;
use chrono::{DateTime, Local};
use colored::*;
use std::path::PathBuf;

pub fn log_command(count: usize) -> Result<()> {
    let mut current_commit_hash = get_current_commit()?;

    if current_commit_hash.is_none() {
        println!("{}", "No commits yet.".yellow());
        return Ok(());
    }

    println!("{}", "Commit History".bold().blue());
    println!("{}", "=".repeat(50).blue());

    let mut commits_shown = 0;
    while let Some(commit_hash) = current_commit_hash {
        if commits_shown >= count {
            break;
        }

        let commit = Commit::load(&commit_hash, &PathBuf::from(&*OBJ_DIR))?;
        print_commit(&commit_hash, &commit, commits_shown == 0);

        current_commit_hash = commit.parent;
        commits_shown += 1;
    }

    if commits_shown >= count {
        println!(
            "\n{}",
            format!("... and {} more commits", commits_shown).dimmed()
        );
    }

    Ok(())
}

fn print_commit(hash: &str, commit: &Commit, is_latest: bool) {
    let local_date: DateTime<Local> = commit.timestamp.with_timezone(&Local);
    let formatted_date = local_date.format("%Y-%m-%d %H:%M:%S %z");

    println!("{}", "┌".yellow());
    if is_latest {
        println!("{}  {}", "│".yellow(), "HEAD -> main".green());
    }
    println!(
        "{}  {} {}",
        "│".yellow(),
        "commit".yellow(),
        hash.bright_yellow()
    );
    println!("{}  {} {}", "│".yellow(), "Author:".cyan(), commit.author);
    println!("{}  {} {}", "│".yellow(), "Date:".cyan(), formatted_date);
    println!("{}", "│".yellow());
    for line in commit.message.lines() {
        println!("{}      {}", "│".yellow(), line);
    }
    println!("{}\n", "└".yellow());
}
