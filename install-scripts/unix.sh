#!/bin/bash

# Determine OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

# Set the download URL base
REPO_URL="https://github.com/mikeshootzz/xpctl/releases/download/v1.0.0-alpha.4"

# Determine the appropriate binary based on OS and ARCH
case "$OS" in
  Darwin)
    if [ "$ARCH" == "x86_64" ]; then
      FILENAME="xpctl-x86_64-apple-darwin.tar.gz"
    elif [ "$ARCH" == "arm64" ]; then
      FILENAME="xpctl-aarch64-apple-darwin.tar.gz"
    else
      echo "Unsupported architecture: $ARCH"
      exit 1
    fi
    ;;

  Linux)
    if [ "$ARCH" == "x86_64" ]; then
      FILENAME="xpctl-x86_64-unknown-linux-gnu.tar.gz"
    else
      echo "Unsupported architecture: $ARCH"
      exit 1
    fi
    ;;

  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

# Download and install the binary
URL="$REPO_URL/$FILENAME"

echo "Downloading $URL..."
curl -L -o "$FILENAME" "$URL"

# Extract the binary
tar -xzf "$FILENAME"

# Move the binary to /usr/local/bin (requires sudo)
sudo mv xpctl /usr/local/bin

# Clean up
rm "$FILENAME"

echo "Installation complete!"
echo "Please set the XPCTL_API_KEY environment variable to your API key of XPipe"
