use crate::objects::commit::compare_commits;
use crate::objects::delta::{Delta, DeltaType};
use crate::utils::OBJ_DIR;
use anyhow::Result;
use colored::Colorize;

pub fn text_diff(old: &str, new: &str) -> (String, usize, usize) {
    let mut diff = String::new();
    let changes = diff::lines(old, new);
    let mut added = 0;
    let mut deleted = 0;

    for change in changes {
        match change {
            diff::Result::Left(l) => {
                diff.push_str(&format!("-{}\n", l));
                deleted += 1;
            }
            diff::Result::Both(l, _) => diff.push_str(&format!(" {}\n", l)),
            diff::Result::Right(r) => {
                diff.push_str(&format!("+{}\n", r));
                added += 1;
            }
        }
    }

    (diff, added, deleted)
}

pub fn diff_command(from: Option<String>, to: Option<String>) -> Result<()> {
    let from_commit = from.unwrap_or_else(|| "HEAD~".to_string());
    let to_commit = to.unwrap_or_else(|| "HEAD".to_string());

    let delta = compare_commits(&from_commit, &to_commit, &*OBJ_DIR)?;

    print_delta(&delta);

    Ok(())
}

fn print_delta(delta: &Delta) {
    println!(
        "Delta between {} and {}",
        delta.from.as_deref().unwrap_or("initial").blue(),
        delta.to.as_deref().unwrap_or("working").blue()
    );

    println!("Changes:");
    for (path, file_delta) in &delta.files {
        match file_delta.delta_type {
            DeltaType::Added => println!("{}", format!("  A\t{} (added)", path.display()).green()),
            DeltaType::Deleted => {
                println!("{}", format!("  D\t{} (deleted)", path.display()).red())
            }
            DeltaType::Modified => {
                println!("{}", format!("  M\t{} (modified)", path.display()).yellow());
                println!(
                    "    {} lines added, {} lines deleted",
                    file_delta.added_lines.to_string().green(),
                    file_delta.deleted_lines.to_string().red()
                );
            }
            DeltaType::Renamed => println!(
                "{}",
                format!(
                    "  R\t{} -> {} (renamed)",
                    file_delta.old_path.as_ref().unwrap().display(),
                    file_delta.new_path.as_ref().unwrap().display()
                )
                .cyan()
            ),
        }
    }
}
