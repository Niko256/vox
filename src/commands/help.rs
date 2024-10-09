use anyhow::Result;

pub fn help_command() -> Result<()> {
    println!("Usage: vcs [COMMAND]");
    println!("Commands:");
    println!("  init        Initialize a new vcs repository");
    println!("  cat-file -p <hash>  Display object content with header");
    println!("  help        Display this help message");

    Ok(())
}
