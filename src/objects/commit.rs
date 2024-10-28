use chrono::prelude::*;


#[derive(Debug)]
struct Commit {
    tree_hash: String, 
    parent_hash: Option<String>,
    author: String,
    committer: String,
    message: String,
    timestamp: DateTime<Utc>,
}


impl Commit {
    fn new(tree_hash: String, parent_hash: Option<String>,
        author: String, committer: String,
        message: String, timestamp: DateTime<Utc>) -> Self {
        
        Commit {
            tree_hash,
            parent_hash,
            author,
            committer,
            message,
            timestamp: Utc::now(),
        }
    }
}

