#!/bin/bash

echo "Creating udev rule..."

# 1038 is, from what I can tell, the SteelSeries vendor ID
sudo tee /etc/udev/rules.d/99-linuxmix.rules > /dev/null <<EOF
KERNEL=="hidraw*", ATTRS{idVendor}=="1038", MODE="0644", GROUP=$USER
EOF

echo "Reloading udev rules..."
sudo udevadm control --reload-rules
sudo udevadm trigger

echo "Copying binary to /home/$USER/.local/bin/..."
mkdir -p ~/.local/bin/
cp target/release/linuxmix ~/.local/bin/linuxmix

echo "Generating service file /home/$USER/.config/systemd/user/linuxmix.service"

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
