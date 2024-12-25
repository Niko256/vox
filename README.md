# Version Control System (VCS)

A lightweight version control system implemented in Rust, inspired by Git. This educational project aims to provide deep insights into how version control systems work internally.

## Features

### Basic commands
- `vcs init` - Initialize a new repository
- `vcs status` - Show working tree status
- `vcs add <files>` - Add files to the staging area (Index)
- `vcs commit -m "message"` - Record changes to the repository
- `vcs log` - Show commit history

### Object Managment
- `vcs hash-object <file>` - Compute object ID and optionally creates a blob
- `vcs cat-file` - Provide content or type and size information for repository objects
- `vcs write-tree` - Create a tree object from the current index
- `vcs show [commit]` - Show various types of objects with detailed information

### Index Operations
- `vcs ls-files` - Show information about files in the index
- `vcs rm [--cached] files` - Remove filesfrom working tree and index

### Repository Information
- `vcs branch` - List, create, or delete branches
- `vcs status` - Show working tree status

## Installation

1. Clone the repository:

```bash
https://github.com/Niko256/vcs.git
```
2. Run the installaton script

```bash
./install.sh
```
The install.sh script will guide you through the installation or update process.

## Contributing
Contributions are welcome! Please feel free to submit a Pull Request.
