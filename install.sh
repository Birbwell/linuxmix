#!/bin/bash

ss_vendor_id=1038

echo "Checking for binary..."
if [ ! -e "./target/release/linuxmix" ] && command -v cargo &> /dev/null; then      # Check if binary is present and if cargo is installed
    cargo build --release
elif [ ! -e "Cargo.toml" ]; then                                                    # Check if this is a rust project
    echo "Binary is not present, and this is not a rust project. Exiting."
    exit 1
elif [ ! -e "./target/release/linuxmix" ] && ! command -v cargo &> /dev/null; then  # Check if cargo is not installed
    echo "Binary is not present, and cargo is not installed. Exiting."
    exit 1
fi

echo "Creating udev rule..."

# 1038 is, from what I can tell, the SteelSeries vendor ID
sudo tee /etc/udev/rules.d/99-linuxmix.rules > /dev/null <<EOF
KERNEL=="hidraw*", ATTRS{idVendor}=="$ss_vendor_id", MODE="0640", GROUP="$USER"
EOF

if ! command -v pactl &> /dev/null; then
    echo "PulseAudio not installed"
    exit 1
fi

if ! command -v pw-cli &> /dev/null; then
    echo "PipeWire not installed"
    exit 1
fi

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
WorkingDirectory=/home/$USER/.config/linuxmix

[Install]
WantedBy=default.target
EOF

echo "Generating service configuration directory /home/$USER/.config/linuxmix/"
mkdir ~/.config/linuxmix

echo "Starting service..."
systemctl --user daemon-reload
systemctl --user enable linuxmix
systemctl --user start linuxmix

echo "Done!"
echo "Mess with the ChatMix dial to ensure the service works"
echo "If the device was not plugged in when installing the service, restart the service after plugging in the device by running 'systemctl --user restart linuxmix'"
