# Vox

A lightweight version control system implemented in Rust, inspired by Git. This educational project aims to provide deep insights into how version control systems work internally.

## Features

### Basic commands
- `vox init` - Initialize a new repository
- `vox status` - Show working tree status
- `vox add <files>` - Add files to the staging area (Index)
- `vox commit -m "message"` - Record changes to the repository
- `vox log` - Show commit history

### Object Managment
- `vox hash-object <file>` - Compute object ID and optionally creates a blob
- `vox cat-file` - Provide content or type and size information for repository objects
- `vox write-tree` - Create a tree object from the current index
- `vox show [commit]` - Show various types of objects with detailed information

### Index Operations
- `vox ls-files` - Show information about files in the index
- `vox rm [--cached] files` - Remove filesfrom working tree and index

### Repository Information
- `vox branch` - List, create, or delete branches
- `vox status` - Show working tree status

## Installation

1. Clone the repository:

```bash
https://github.com/Niko256/vox.git
```
2. Run the installaton script

```bash
./install.sh
```
The install.sh script will guide you through the installation or update process.
