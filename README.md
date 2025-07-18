# linuxmix
A simple interface service to interact with SteelSeries ChatMix dials on linux

I'm sure there are some other alternatives out there. The two I had seen prior to starting this required installing dependencies. My goal with linuxmix was to get the SteelSeries ChatMix feature working while using as few dependencies as possible. As a result, the only requirement to get this working _as far as I am aware_ is to be using Pulse Audio as your sound server.

### Notes

- This program uses the Pulse Audio server to configure the audio sinks. This will not work if you are using an alternate server, as it uses pactl to create/remove audio sinks. I'm not against adding support for other ones, I just am starting with the one I am actively using.
- This has been primarily tested using Fedora 42 Workstation. I don't believe there will be any issues if it is run on other distros, but it is just something to keep in mind.

### How to Install
1) By downloading the release
    - Download the latest release
    - Open a console in the directory the tar archive is in
    - Run:
    ```bash
    mkdir linuxmix/ && tar -xvzf linuxmix.tar.gz -C linuxmix/
    cd linuxmix
    bash install.sh
    ```

2) By building from source
   - Run:
   ```bash
   git clone https://github.com/Birbwell/linuxmix.git
   cd linuxmix
   cargo build --release
   bash install.sh
   ```

When you're done installing, make sure your main output sink is set to be `Game`.

### How to Uninstall
Run `bash uninstall.sh`

I am **VERY** new to making an open-source project, so any tips/contributions/recommendations are welcome!

I will say, though the GitHub workflow is currently failing, the program works as expected. I'm just still working on configuring it to automatically create releases lol (again, I am very new).
