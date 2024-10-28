use clap::Parser;
use anyhow::Result;
use crate::objects::blob::create_blob;


#[derive(Parser, Debug)]
pub struct HashObjectArgs {
    pub file_path: String,
}


pub fn hash_object_command(args: HashObjectArgs) -> Result<()> {
    let object_hash = create_blob(&args.file_path)?;
    println!("{}", object_hash);
    Ok(())
}


