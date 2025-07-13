# linuxmix
A simple interface service to interact with SteelSeries ChatMix dials on linux

### Notes

- This program uses the Pulse Audio server to configure the audio sinks. This may not work if you are using an alternate server. I'm not against adding support for other ones, I just am starting with the one I am actively using.
- This has been primarily tested using Fedora 42. I don't believe there will be any issues if it is run on other distros (as long as the distro uses Pulse Audio!), but it is just something to keep in mind.

### How to Install
1) By downloading the release
    Extract linuxmix.tar.gz
    Run `./install.sh`

2) By cloning the repository
    Clone the repository
    Run `cargo build --release` (must have cargo/rust installed)
    Run `./install.sh`

### How to Uninstall
Run `./uninstall.sh`

I am **VERY** new to making an open-source project, so any tips/contributions/recommendations are welcome!

I will say, though the GitHub workflow is currently failing, the program works as expected. I'm just still working on configuring it to automatically create releases lol (again, I am very new).
