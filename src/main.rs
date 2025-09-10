use std::{
    fs::{File, OpenOptions},
    io::Read,
    process::{Command, Stdio},
    sync::Mutex,
    thread, time,
};

const CHATMIX_CODE: u8 = 69;        // Opcode for chatmix signal
const HEADSET_POWER: u8 = 185;      // Opcode for power
const GAME: &str = "Game";
const CHAT: &str = "Chat";

fn main() -> ! {
    let devices = get_devices();
    let mut threads = vec![];
    for device in devices {
        threads.push(thread::spawn(|| read_device(device)));
    }

    while threads.iter().any(|f| !f.is_finished()) {
        thread::sleep(time::Duration::from_millis(100));
    }

    panic!("All threads exited unexpectedly");
}

fn get_devices() -> Vec<File> {
    std::fs::read_dir("/dev/")
        .unwrap()
        .filter_map(|f| {
            let t = f.unwrap().file_name().into_string().unwrap();
            if t.starts_with("hidraw") {
                if let Ok(dev) = OpenOptions::new().read(true).open(format!("/dev/{t}")) {
                    Some(dev)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<File>>()
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

fn configure_sinks() {
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

    let load_null_sink = |sink_name: &str| {
        Command::new("pactl")
            .arg("load-module")
            .arg("module-null-sink")
            .arg(format!("sink_name={sink_name}"))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    };

    let load_loopback = |source: &str| {
        Command::new("pactl")
            .arg("load-module")
            .arg("module-loopback")
            .arg(format!("source={source}.monitor"))
            .arg(format!("sink={default_sink}"))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    };

    load_null_sink(GAME);
    load_null_sink(CHAT);

    load_loopback(GAME);
    load_loopback(CHAT);
}

fn read_device(mut file: File) {
    let mut buf = [8u8; 4];
    while let Ok(()) = file.read_exact(&mut buf) {
        process_bytes(buf);
    }
}

fn process_bytes([code, game_vol, chat_vol, _]: [u8; 4]) {
    static IS_CONF: Mutex<bool> = Mutex::new(false);

    let set_volume = |channel: &str, vol: u8| {
        Command::new("pactl")
            .arg("set-sink-volume")
            .arg(format!("{}", channel))
            .arg(format!("{}%", vol))
            .spawn()
            .unwrap();
    };

    match code {
        CHATMIX_CODE => {
            if let Ok(mut conf) = IS_CONF.lock()
                && !*conf
            {
                configure_sinks();
                *conf = true;
            }
            set_volume(GAME, game_vol);
            set_volume(CHAT, chat_vol)
        }
        HEADSET_POWER if game_vol == 2 => {
            // Power off
            if let Ok(mut conf) = IS_CONF.lock()
                && *conf
            {
                cleanup_sinks();
                *conf = false;
            }
            return; // Return early bc we dont want to set the default sink to anything
        }
        HEADSET_POWER if game_vol == 3 => {
            // Power on
            if let Ok(mut conf) = IS_CONF.lock()
                && !*conf
            {
                configure_sinks();
                *conf = true;
            }
        }
        _ => return, // Do nothing if otherwise
    }

    Command::new("pactl")
        .arg("set-default-sink")
        .arg("Game")
        .spawn()
        .unwrap();
}

fn cleanup_sinks() {
    Command::new("pactl")
        .arg("unload-module")
        .arg("module-loopback")
        .stdout(Stdio::null())
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
