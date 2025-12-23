# Check the Linux distro we're running:
cat /etc/os-release

# Add cargo to the path both temporarily and permanently:
export PATH="$HOME/.cargo/bin:$PATH"
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.profile

# Ensure cargo command is available
command -v cargo

# Install odjitter using cargo
cargo install --git https://github.com/dabreegster/odjitter --rev 32fb58bf7f0d68afd3b76b88cf6b1272c5c66828
# Add local instance of odjitter to the /usr/local/bin directory:
which odjitter
sudo ln -s ~/.cargo/bin/odjitter /usr/local/bin/odjitter

# Ensure R is installed and execute the R script
Rscript code/install.R

# Ensure apt repository is up-to-date and install Python packages
apt-get update
apt-get install -y software-properties-common python3 python3-pip

# Install Python dependencies:
pip install -r requirements.txt

# Clone and install tippecanoe if not already installed
cd /tmp
if [ ! -d "tippecanoe" ]; then
    git clone https://github.com/felt/tippecanoe.git
fi
cd tippecanoe
make -j$(nproc)
sudo make install
tippecanoe --version

# Install git and GitHub CLI (gh)
apt-get install -y git
apt-get install -y gh

# Configure git settings
git config --global core.autocrlf input
git config --global core.fileMode false
# Remove any stale proxy settings that may block Cargo
git config --global --unset http.proxy 2>/dev/null || true
git config --global --unset https.proxy 2>/dev/null || true
git update-index --refresh

# Configure TeX Live to use the frozen 2024 repository (avoids version mismatch errors)
tlmgr option repository https://ftp.math.utah.edu/pub/tex/historic/systems/texlive/2024/tlnet-final
tlmgr update --self
# Install LaTeX packages needed for Quarto PDF rendering
tlmgr install unicode-math fontspec l3packages

# Make sure there's a newline at the end of the script
echo "Script execution completed successfully."
