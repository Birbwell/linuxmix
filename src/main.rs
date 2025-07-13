use std::{
    fs::{File, OpenOptions},
    io::Read,
    process::{Command, Stdio},
};

const CHATMIX_CODE: u8 = 69;
const HEADSET_POWER: u8 = 185;
const GAME: &str = "Game";
const CHAT: &str = "Chat";

enum State {
    Running,
    Paused,
    Waiting,
}

fn main() {
    let mut stream = if let Ok(device) = std::fs::read_to_string("device.conf") {
        OpenOptions::new().read(true).open(format!("/dev/{device}")).unwrap()
    } else {
        determine_device()
    };

    let mut state = State::Waiting;

    loop {
        match state {
            State::Running => {
                loop {
                    let mut bytes = [0u8; 4];
                    stream.read_exact(&mut bytes).unwrap();
                    if process_bytes(bytes) {
                        break;
                    }
                }
                cleanup_sinks();
                state = State::Paused;
            }
            State::Paused => {
                let mut bytes = [0u8; 4];
                stream.read_exact(&mut bytes).unwrap();
                if bytes[0] == 185 && bytes[1] == 3 {
                    configure_device();
                    state = State::Running;
                }
            }
            State::Waiting => {
                configure_device();
                state = State::Running;
            }
        }
    }
}

fn determine_device() -> File {
    println!("Mess around with the chatmix dial so the service can determine the correct file to read from!");

    let mut devices = std::fs::read_dir("/dev/").unwrap().filter_map(|f| {
        let t = f.unwrap().file_name().into_string().unwrap();
        if t.starts_with("hidraw") {
            let dev = OpenOptions::new()
                .read(true)
                .open(format!("/dev/{t}"))
                .ok()?;
            Some((dev, t))
        } else {
            None
        }
    });

    let (determined_device, hidraw_name) = 'device_loop: loop {
        for (mut dev, name) in &mut devices {
            let mut buf = [0u8; 4];
            if let Ok(4) = dev.read(&mut buf) {
                if let [CHATMIX_CODE, _, _, 0] = buf {
                    break 'device_loop (dev, name);
                }
            }
        }
    };

    std::fs::write("device.conf", hidraw_name).unwrap();

    determined_device
}

fn configure_device() {
    let default_sink = {
        let default_sink_name = loop {
            let def_sink_out = Command::new("pactl")
                .arg("get-default-sink")
                .output()
                .unwrap()
                .stdout;

            if !def_sink_out.is_empty() {
                break def_sink_out;
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        };

        get_device_id(&String::from_utf8(default_sink_name).unwrap().trim())
    };

    // Create Game sink
    Command::new("pactl")
        .arg("load-module")
        .arg("module-null-sink")
        .arg(format!("sink_name={}", GAME))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    // Create Chat sink
    Command::new("pactl")
        .arg("load-module")
        .arg("module-null-sink")
        .arg(format!("sink_name={}", CHAT))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    // Combine them, then loop it to the hadphones
    Command::new("pactl")
        .arg("load-module")
        .arg("module-loopback")
        .arg(format!("source={CHAT}.monitor"))
        .arg(format!("sink={default_sink}"))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    Command::new("pactl")
        .arg("load-module")
        .arg("module-loopback")
        .arg(format!("source={GAME}.monitor"))
        .arg(format!("sink={default_sink}"))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn process_bytes([code, game_vol, chat_vol, _]: [u8; 4]) -> bool {
    match code {
        // 69 is for volume wheel values
        CHATMIX_CODE => {
            Command::new("pactl")
                .arg("set-sink-volume")
                .arg(format!("{}", GAME))
                .arg(format!("{}%", game_vol))
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            Command::new("pactl")
                .arg("set-sink-volume")
                .arg(CHAT)
                .arg(format!("{}%", chat_vol))
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
        HEADSET_POWER if game_vol == 2 => {
            //power_off
            return true;
        }
        _ => {} // Other opcodes are not implemented, but we dont want this program to crash
    }
    return false;
}

fn get_device_id(device_name: &str) -> u32 {
    let mut pactl_out = Command::new("pactl")
        .arg("list")
        .arg("sinks")
        .arg("short")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    pactl_out.wait().unwrap();

    let grep_out = Command::new("grep")
        .arg(device_name)
        .stdin(Stdio::from(pactl_out.stdout.take().unwrap()))
        .output()
        .unwrap()
        .stdout;

    String::from_utf8(grep_out)
        .unwrap()
        .split_ascii_whitespace()
        .nth(0)
        .and_then(|sink_code| sink_code.parse::<u32>().ok())
        .unwrap()
}

fn cleanup_sinks() {
    Command::new("pactl")
        .arg("unload-module")
        .arg("module-loopback")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    Command::new("pw-cli")
        .arg("destroy")
        .arg(GAME)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    Command::new("pw-cli")
        .arg("destroy")
        .arg(CHAT)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
