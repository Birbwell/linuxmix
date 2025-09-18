# linuxmix
A simple interface service to interact with SteelSeries ChatMix dials on linux

I'm sure there are some other alternatives out there. The two I had seen prior to starting this required installing dependencies. My goal with linuxmix was to get the SteelSeries ChatMix feature working while using as few dependencies as possible. As a result, the only requirement to get this working _as far as I am aware_ is to be using Pulse Audio as your sound server.

### Notes
- This program uses the Pulse Audio server to configure the audio sinks. This will not work if you are using an alternate server, as it uses pactl to create/remove audio sinks. I'm not against adding support for other ones, I just am starting with the one I am actively using.
- This has been primarily tested using Fedora 42 Workstation and CachyOS. I don't believe there will be any issues if it is run on other distros, but it is just something to keep in mind.
- I've considered adding this to a repository such as the AUR, however since it is required to run in user-mode (not as root), it is difficult to get it properly set-up through those methods. AUR packages generally should not touch anything in the /home/ directory (I know it's technically possible, but I don't want to make anyone mad lol).

### How to Install
1) Running `curl https://raw.githubusercontent.com/Birbwell/linuxmix/refs/heads/main/install.sh | bash`
   - Requires Git and Cargo (rust) installed
   - Clones, compiles, and configures the binary using the latest source code

2) By downloading the release
    - Download the latest release
    - Run in the terminal:
    ```bash
    mkdir linuxmix/ && tar -xvzf linuxmix.tar.xz -C linuxmix/
    cd linuxmix
    bash install.sh
    ```

3) By building from source
   - If the other two do not work, you can clone the repository and build it manually
   - Requires git and cargo to complete
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

I am still **VERY** new to making open-source projects, so any tips/contributions/recommendations are welcome! Please do not hesitate to report any issues, as I want to make this as bug-free as possible!
