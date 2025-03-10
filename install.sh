#!/bin/bash

cd "$(dirname "$0")"

# Check if Rust and Cargo are installed
if ! command -v rustc &> /dev/null || ! command -v cargo &> /dev/null; then
    echo "Rust and Cargo are required but not installed. Please install them first."
    exit 1
fi

# Function to install the project
install_project() {
    echo "Installing vox..."
    cargo install --path .
    if [ $? -eq 0 ]; then
        echo "vox installed successfully."
        add_to_path
    else
        echo "Failed to install vox."
        exit 1
    fi
}

# Function to update the project
update_project() {
    echo "Updating vox..."
    cargo install --path . --force
    if [ $? -eq 0 ]; then
        echo "vox updated successfully."
        add_to_path
    else
        echo "Failed to update vox."
        exit 1
    fi
}

# Function to add vox to the PATH
add_to_path() {
    echo "Adding vox to PATH..."
    if [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
        echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
        echo "Please restart your shell or run 'source ~/.bashrc' to apply changes."
    else
        echo "vox is already in PATH."
    fi
}

echo "Welcome to the vox installer!"
echo "Please select an option:"
echo "1. Install vox"
echo "2. Update vox"
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
