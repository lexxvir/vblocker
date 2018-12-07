use rustc_serialize::json;
use std::{
    env::args, fs::remove_file, fs::File, io::prelude::*, process::Command, thread::sleep, time::Duration,
};

static TIMEHOLDER_PATH: &str = "timeholder";
static PROCESSES: &[&str] = &[ "vlc", "smplayer", "gnome-mplayer", "totem" ] /*[ "sleep" ]*/;
static UPDATE_TIME: u64 = 5;

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Times {
    run: u64,
    idle: u64,
}

fn load_times() -> Times {
    let mut file = match File::open(TIMEHOLDER_PATH) {
        Ok(f) => f,
        Err(_) => return Times { run: 0, idle: 0 },
    };

    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    return json::decode(&content).unwrap();
}

fn save_times(times: &Times) {
    let string = json::encode(times).unwrap();
    let mut file = File::create(TIMEHOLDER_PATH).unwrap();

    file.write_all(string.as_bytes()).unwrap();
}

fn is_exist(name: &str) -> bool {
    let mut cmd = Command::new("pgrep");
    cmd.arg(name);

    cmd.output().unwrap().status.success()
}

fn check_processes() -> bool {
    for p in PROCESSES.iter() {
        if is_exist(*p) {
            return true;
        }
    }
    return false;
}

fn kill_processes() {
    println!("kill!");

    for p in PROCESSES.iter() {
        let mut cmd = Command::new("pkill");
        cmd.arg(*p);
        let _ = cmd.output();
    }
}

fn worker(allow_time: u64, deny_time: u64) {
    loop {
        sleep(Duration::from_secs(UPDATE_TIME));

        let mut times = load_times();
        if check_processes() {
            times.run += UPDATE_TIME;
        } else {
            times.idle += UPDATE_TIME;
        }

        save_times(&times);

        if times.run > allow_time && times.idle < deny_time {
            kill_processes();
        } else if times.idle >= deny_time {
            remove_file(TIMEHOLDER_PATH).unwrap();
        }
    }
}

fn main() {
    if args().len() < 3 {
        println!("Usage: video_blocker <allow time> <deny time>, all times in seconds");
        return;
    }

    let allow_time: u64 = args().nth(1).and_then(|x| x.parse().ok()).unwrap_or(0);
    let deny_time: u64 = args().nth(2).and_then(|x| x.parse().ok()).unwrap_or(0);

    println!(
        "Allow time is {} s, Deny time is {} s",
        allow_time, deny_time
    );

    worker(allow_time, deny_time);
}
