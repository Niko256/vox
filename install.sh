#!/bin/bash

cd "$(dirname "$0")"

# Check if Rust and Cargo are installed
if ! command -v rustc &> /dev/null || ! command -v cargo &> /dev/null; then
    echo "Rust and Cargo are required but not installed. Please install them first."
    exit 1
fi

# Function to install the project
install_project() {
    echo "Installing vcs..."
    cargo install --path .
    if [ $? -eq 0 ]; then
        echo "vcs installed successfully."
    else
        echo "Failed to install vcs."
        exit 1
    fi
}

# Function to update the project
update_project() {
    echo "Updating vcs..."
    cargo install --path . --force
    if [ $? -eq 0 ]; then
        echo "vcs updated successfully."
    else
        echo "Failed to update vcs."
        exit 1
    fi
}

echo "Welcome to the vcs installer!"
echo "Please select an option:"
echo "1. Install vcs"
echo "2. Update vcs"
echo "3. Exit"

read -p "Enter your choice (1/2/3): " choice

case $choice in
    1)
        install_project
        ;;
    2)
        update_project
        ;;
    3)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo "Invalid choice. Exiting..."
        exit 1
        ;;
esac
