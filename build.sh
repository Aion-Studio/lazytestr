
#!/bin/sh

# Name of the binary
BINARY_NAME="lazytestr"

# Directory to move the binary to
DEST_DIR="/usr/local/bin"

# Function to check if a directory is in the PATH
is_in_path() {
    echo "$PATH" | tr ':' '\n' | grep -qx "$1"
}

# Build the project
cargo build --release

# Check if the build was successful
if [ $? -ne 0 ]; then
    echo "Build failed. Exiting."
    exit 1
fi

# Check if /usr/local/bin is in the PATH
if ! is_in_path "$DEST_DIR"; then
    echo "$DEST_DIR is not in your PATH. Please add it and try again."
    exit 1
fi

# Move the binary to /usr/local/bin
sudo mv "target/release/$BINARY_NAME" "$DEST_DIR/"

# Check if the move was successful
if [ $? -ne 0 ]; then
    echo "Failed to move the binary. Exiting."
    exit 1
fi

echo "Binary moved to $DEST_DIR successfully."
