use crate::objects::commit::compare_commits;
use crate::objects::delta::{Delta, DeltaType};
use crate::utils::OBJ_DIR;
use anyhow::{Context, Result};
use colored::Colorize;
use similar::{ChangeTag, TextDiff};

/// Computes the unified diff using the Mayers algorithm
///
/// # Arguments
///
/// * 'old' - The old version of the text
/// * 'new' - The new version of the text
///
/// # Returns
///
/// A tuple containing:
/// - The unified diff text
/// - Number of insertions
/// - Number of deletions
///
pub fn text_diff(old: &str, new: &str) -> (String, usize, usize) {
    let diff = TextDiff::configure()
        .algorithm(similar::Algorithm::Myers)
        .diff_lines(old, new);

    let mut unified_diff = String::new();
    let mut insertions = 0;
    let mut deletions = 0;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => {
                unified_diff.push_str(&format!("-{}\n", change));
                deletions += 1;
            }
            ChangeTag::Insert => {
                unified_diff.push_str(&format!("+{}\n", change));
                insertions += 1;
            }
            ChangeTag::Equal => {
                unified_diff.push_str(&format!(" {}\n", change));
            }
        }
    }

    (unified_diff, insertions, deletions)
}

/// Show difference between two commits or workdir states
///
/// # Arguments
///
/// * 'from' - source commit/reference (default: HEAD~)
/// * 'to' - target commit/reference (default: HEAD)
///
///  # Examples
///
///  A <- B <- C <- D
///  where:
///     'HEAD' = commit D
///     'HEAD~' = commit C
///     'HEAD~2' = commit B
///     'HEAD~3' = commit A
///
/// diff_command(None, None).unwrap(); => comparison between 'HEAD~' and 'HEAD'
///
pub fn diff_command(from: Option<String>, to: Option<String>) -> Result<()> {
    let from_ref = from.as_deref().unwrap_or("HEAD~");
    let to_ref = to.as_deref().unwrap_or("HEAD");

    let delta = compare_commits(from_ref, to_ref, &*OBJ_DIR)
        .with_context(|| format!("Failed to compare commits {}..{}", from_ref, to_ref))?;

    print_delta(&delta).context("Failed to display diff output")?;

    Ok(())
}

/// Prints the delta in human-readable format
///
/// # Arguments
///
/// * 'delta' - The delta to display
///
fn print_delta(delta: &Delta) -> Result<()> {
    println!(
        "Delta between {} and {}",
        delta.from().unwrap_or("initial").yellow(),
        delta.to().unwrap_or("working").blue()
    );

    if delta.get().is_empty() {
        println!("{}", "No changes".dimmed());
        return Ok(());
    } else {
        println!("Changes: ");
        for (_path, delta_type) in &delta.get() {
            match delta_type {
                DeltaType::ADDED { path, .. } => {
                    println!("{} {}", "A".green(), path.display());
                }
                DeltaType::DELETED { path, .. } => {
                    println!("{} {}", "D".red(), path.display());
                }
                DeltaType::MODIFIED {
                    path,
                    old_hash: _,
                    new_hash: _,
                    summary,
                } => {
                    println!("{} {}", "M".yellow(), path.display());

                    if let Some(summary) = summary {
                        println!(
                            "  {} lines added, {} lines deleted",
                            summary.insertions().to_string().green(),
                            summary.removals().to_string().red()
                        );
                        if let Some(text_diff) = summary.text_diff() {
                            println!("{}", text_diff);
                        }
                    }
                }
                DeltaType::RENAMED {
                    old_path,
                    new_path,
                    old_hash: _,
                    new_hash: _,
                    summary,
                } => {
                    println!(
                        "{} {} -> {}",
                        "R".cyan(),
                        old_path.display(),
                        new_path.display()
                    );

                    if let Some(summary) = summary {
                        println!(
                            "{} lines added, {} lines deleted",
                            summary.insertions().to_string().green(),
                            summary.removals().to_string().red()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
