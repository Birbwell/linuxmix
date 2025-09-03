use std::{
    fs::{File, OpenOptions},
    io::Read,
    process::{Command, Stdio},
    thread, time,
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

#[derive(PartialEq, Eq)]
enum StreamState {
    Continue,
    Invalid,
    Break,
}

fn main() {
    let read_bytes = |stream: &mut File, uninitiated: bool| {
        let mut bytes = [0u8; 4];
        stream.read_exact(&mut bytes).unwrap();
        process_bytes(bytes, uninitiated)
    };

    let mut stream = loop {
        let opt_stream = if let Ok(device) = std::fs::read_to_string("device.conf") {
            OpenOptions::new()
                .read(true)
                .open(format!("/dev/{device}"))
                .ok()
                .or(determine_device())
        } else {
            determine_device()
        };

        if let Some(_stream) = opt_stream {
            break _stream;
        }

        let Some(mut _stream) = opt_stream else {
            eprintln!("Did not receive ChatMix dial signal. Retrying...");
            thread::sleep(time::Duration::from_secs(2));
            continue;
        };

        // Confirm whether or not the file read is valid
        // This will confirm if it is reading the correct file, as sometimes one of the three SteelSeries HIDRAW devices
        // can transmit data, leading to the service continuing without properly using the correct HIDRAW device
        let stat = read_bytes(&mut _stream, true);
        if stat != StreamState::Invalid {
            break _stream;
        }
    };

    let mut state = State::Waiting;

    loop {
        match state {
            State::Running => {
                loop {
                    let stat = read_bytes(&mut stream, false);
                    if stat == StreamState::Break {
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

fn determine_device() -> Option<File> {
    // Putting this here so its viewable with systemctl status
    println!(
        "Mess around with the chatmix dial so the service can determine the correct file to read from!"
    );

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

    let now = time::Instant::now();
    let timeout = time::Duration::from_secs(10);

    let (determined_device, hidraw_name) = 'device_loop: loop {
        if now.elapsed() < timeout {
            for (mut dev, name) in &mut devices {
                let mut buf = [0u8; 4];
                if let Ok(4) = dev.read(&mut buf) {
                    if let [CHATMIX_CODE, _, _, 0] = buf {
                        break 'device_loop (dev, name);
                    }
                }
            }
        } else {
            return None;
        }
    };

    std::fs::write("device.conf", hidraw_name).unwrap();

    Some(determined_device)
}

fn configure_device() {
    // Prevent creating duplicate sinks, which would otherwise happen if the service is abruptly restarted
    cleanup_sinks();

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

            thread::sleep(time::Duration::from_secs(1));
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

fn process_bytes([code, game_vol, chat_vol, _]: [u8; 4], uninitiated: bool) -> StreamState {
    let set_volume = |channel: &str, vol: u8| {
        Command::new("pactl")
            .arg("set-sink-volume")
            .arg(format!("{}", channel))
            .arg(format!("{}%", vol))
            .spawn()
            .unwrap();
    };

    match code {
        // 69 is for volume wheel values
        CHATMIX_CODE => {
            set_volume(GAME, game_vol);
            set_volume(CHAT, chat_vol)
        }
        HEADSET_POWER if game_vol == 2 => {
            //power_off
            return StreamState::Break;
        }
        _ if uninitiated => {
            return StreamState::Invalid;
        }
        _ => {} // Do nothing if otherwise
    }

    Command::new("pactl")
        .arg("set-default-sink")
        .arg("Game")
        .spawn()
        .unwrap();
    return StreamState::Continue;
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

    let destroy_sinks = |name: &str| {
        loop {
            let stat = Command::new("pw-cli")
                .arg("destroy")
                .arg(name)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();

            // Prints to stderr whenever an error occurs, such as there being no sink by the given name
            // Despite printing to stderr, _the exit status is still 0_
            if stat.stderr.len() > 0 {
                break;
            }
        }
    };

    destroy_sinks(GAME);
    destroy_sinks(CHAT);
}
