use anyhow::Result;

pub fn help_command() -> Result<()> {
    println!("Usage: vcs [COMMAND]");
    println!("Commands:");
    println!("  init        Initialize a new vcs repository");
    println!("  cat-file -p <hash>  Display object content with header");
    println!("  hash-object <file_path> Compute the hash of a file and store it in the object database");
    println!("  help        Display this help message");

    Ok(())
}
