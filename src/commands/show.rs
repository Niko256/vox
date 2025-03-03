use crate::commands::commit::get_current_commit;
use crate::objects::commit_object::Commit;
use crate::objects::tree_object::read_tree;
use crate::utils::OBJ_DIR;
use anyhow::Result;
use chrono::{DateTime, Local};
use colored::*;
use std::path::PathBuf;

pub fn show_command(commit_ref: &str) -> Result<()> {
    let commit_hash = if commit_ref == "HEAD" {
        get_current_commit()?.ok_or_else(|| anyhow::anyhow!("No commits yet!"))?
    } else {
        commit_ref.to_string()
    };

    let commit = Commit::load(&commit_hash, &PathBuf::from(&*OBJ_DIR))?;
    print_commit_details(&commit_hash, &commit)?;

    Ok(())
}

fn print_commit_details(hash: &str, commit: &Commit) -> Result<()> {
    let local_date: DateTime<Local> = commit.timestamp.with_timezone(&Local);
    let formatted_date = local_date.format("%Y-%m-%d %H:%M:%S %z");

    println!("{}", "=".repeat(70).blue());
    println!("{} {}", "Commit:".yellow(), hash.bright_purple());
    println!("{} {}", "Author:".cyan(), commit.author);
    println!("{} {}", "Date:".cyan(), formatted_date);
    println!("\n{}", commit.message.bright_white());
    println!("{}", "=".repeat(70).blue());

    println!("\n{}", "Changes:".green().bold());
    print_tree_info(&commit.tree, "", true)?;

    if let Some(parent) = &commit.parent {
        println!("\n{}", "Parent commit:".yellow());

        let parent_commit = Commit::load(parent, &PathBuf::from(&*OBJ_DIR))?;
        println!(
            "  {} {}",
            parent[..8].bright_purple(),
            parent_commit.message.split('\n').next().unwrap_or("")
        );
    }

    Ok(())
}

fn print_tree_info(tree_hash: &str, prefix: &str, _is_last: bool) -> Result<()> {
    let tree = read_tree(tree_hash)?;
    let entries = tree.entries;

    for (idx, entry) in entries.iter().enumerate() {
        let is_last_entry = idx == entries.len() - 1;
        let branch = if is_last_entry {
            "└── "
        } else {
            "├── "
        };
        let next_prefix = if is_last_entry { "    " } else { "│   " };

        // Разные цвета для разных типов файлов
        let display = match entry.object_type.as_str() {
            "tree" => entry.name.blue(),
            "blob" => entry.name.normal(),
            _ => entry.name.red(),
        };

        println!(
            "{}{}{}    {}",
            prefix,
            branch.purple(),
            display,
            format!("[{}]", &entry.object_hash[..8]).dimmed()
        );

        if entry.object_type == "tree" {
            print_tree_info(
                &entry.object_hash,
                &format!("{}{}", prefix, next_prefix),
                is_last_entry,
            )?;
        }
    }
    Ok(())
}
