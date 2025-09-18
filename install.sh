#!/bin/bash

repo_url=https://github.com/Birbwell/linuxmix.git
ss_vendor_id=1038
set -e

echo "Checking for binary..."
if [ ! -f "target/release/linuxmix" ] && [ -f "Cargo.toml" ] && (command -v cargo &> /dev/null); then               # Check if binary is present and if cargo is installed
    cargo build --release
elif [ ! -f "target/release/linuxmix" ] && (command -v git &> /dev/null) && (command -v cargo &> /dev/null); then   # Check if git and cargo is installed, clone and compile if they both are
    echo "Cloning repo..."
    git clone $repo_url /tmp/linuxmix
    cd /tmp/linuxmix
    cargo build --release
elif [ ! -f "target/release/linuxmix" ]; then                                                                       # If the binary cannot be found or compiled proceed, exit
    echo "Binary is not present, and cargo and/or git is not installed. Exiting."
    exit 1
fi

if ! command -v pactl &> /dev/null; then
    echo "PulseAudio not installed"
    exit 1
fi

if ! command -v pw-cli &> /dev/null; then
    echo "PipeWire not installed"
    exit 1
fi

echo "Creating udev rule..."

# 1038 is, from what I can tell, the SteelSeries vendor ID
sudo tee /etc/udev/rules.d/99-linuxmix.rules > /dev/null <<EOF
KERNEL=="hidraw*", ATTRS{idVendor}=="$ss_vendor_id", MODE="0640", GROUP="audio"
EOF

echo "Reloading udev rules..."
sudo udevadm control --reload-rules
sudo udevadm trigger

echo "Copying binary to /home/$USER/.local/bin/..."
mkdir -p ~/.local/bin/
cp target/release/linuxmix ~/.local/bin/linuxmix

echo "Generating service file /home/$USER/.config/systemd/user/linuxmix.service"

mkdir -p ~/.config/systemd/user/
tee ~/.config/systemd/user/linuxmix.service > /dev/null <<EOF
[Unit]
Description=Service to split SteelSeries headphone audio tracks programmatically

[Service]
ExecStart=/home/$USER/.local/bin/linuxmix
Restart=always

[Install]
WantedBy=default.target
EOF

echo "Starting service..."
systemctl --user daemon-reload
systemctl --user enable linuxmix
systemctl --user start linuxmix

echo "Cleaning up..."
rm -rf /tmp/linuxmix

echo "Done!"
echo "Mess with the ChatMix dial to ensure the service works"
echo "If the device was not plugged in when installing the service, restart the service after plugging in the device by running 'systemctl --user restart linuxmix'"
