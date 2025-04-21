# Vox

A lightweight version control system, inspired by Git. This educational project aims to provide insights into how version control systems work internally

## Features

### Repository Management
- `vox init` - Initialize a new repository
- `vox status` - Show working tree status

### Staging Area (Index) Operations
- `vox add <paths>` - Add files to the staging area
- `vox rm [--cached] [--force] <paths>` - Remove files from working tree and/or index
- `vox ls-files [--stage]` - Show information about files in the index
- `vox write-tree [--path]` - Create a tree object from the current index

### Object Management
- `vox hash-object <file>` - Compute object ID and optionally creates a blob
- `vox cat-file [-p] [-t] [-s] <object>` - Inspect repository objects
- `vox show <commit>` - Show detailed object information

### Commit History
- `vox commit -m <message> [--author]` - Record changes to the repository
- `vox log [--count]` - Show commit history
- `vox diff [from] [to]` - Show changes between commits

### Branching
- `vox branch [name] [--delete] [--list]` - List, create or delete branches
- `vox checkout <target> [--force]` - Switch branches or restore working tree files

### Configuration
- `vox config [--global] <command>` - Manage configuration settings
- `vox remote <command>` - Manage remote repositories

## Installation

### From Source

1. Clone the repository:
```bash
git clone https://github.com/Niko256/vox.git
cd vox
```

2. Run the installation script:
```bash
./install.sh
```
