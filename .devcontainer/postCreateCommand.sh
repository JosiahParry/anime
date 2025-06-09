#!/bin/bash
# Ensure we are running under Bash
if [ -z "$BASH_VERSION" ]; then
    echo "This script requires Bash, but it was executed with $SHELL. Exiting."
    exit 1
fi

# Set debugging and exit on error
set -euxo pipefail

# Check the Linux distro we're running:
cat /etc/os-release

# Add cargo to the path both temporarily and permanently:
export PATH="$HOME/.cargo/bin:$PATH"
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.profile

# Ensure cargo command is available
command -v cargo

# Ensure apt repository is up-to-date and install Python packages
apt-get update
apt-get install -y software-properties-common python3 python3-pip

# 1. Install required dependencies
sudo apt update
sudo apt install curl git -y

# 2. Download and install GitHub CLI
type -p curl >/dev/null || sudo apt install curl -y
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | \
  sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
sudo chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg

# 3. Add the GitHub CLI repository
echo "deb [arch=$(dpkg --print-architecture) \
signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] \
https://cli.github.com/packages stable main" | \
sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null

# 4. Update and install gh
sudo apt update
sudo apt install gh -y

sudo apt install dos2unix

# Configure git settings
git config --global core.autocrlf input
git config --global core.fileMode false
git update-index --refresh

# Make sure there's a newline at the end of the script
echo "Script execution completed successfully."
