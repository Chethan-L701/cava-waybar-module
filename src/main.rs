use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

fn playerctl_active(plyr_status: &Arc<AtomicBool>) {
    let playerctl = Command::new("playerctl").arg("status").output().unwrap();
    if playerctl.status.success() {
        plyr_status.store(true, Ordering::Relaxed);
        return;
    }
    plyr_status.store(false, Ordering::Relaxed);
}

fn format_bar(config: &String, plyr_status: &Arc<AtomicBool>) -> io::Result<()> {
    let bar: Vec<char> = vec!['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let mut cava = Command::new("cava")
        .arg("-p")
        .arg(config)
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = cava.stdout.take().expect("Faild to capture stdout");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        if plyr_status.load(Ordering::Relaxed) {
            let line = line?;

            let strenth: Vec<u8> = line
                .split(';')
                .filter(|x| !x.is_empty())
                .map(|x| x.parse().unwrap())
                .collect();

            let mut visulizer: String = String::new();

            for val in strenth {
                if let Some(c) = bar.get(val as usize) {
                    visulizer.push(*c);
                }
            }
            println!("{{\"text\" : \"{}\" }}", visulizer);
            io::stdout().flush().unwrap();
        } else {
            println!("{{\"text\" : \"\", \"class\" : \"hidden\"}}");
            io::stdout().flush().unwrap();
        }
    }

    Ok(())
}

fn init_cava_config(confin_path: &String) {
    let config = r#"
        [general]
        mode = normal
        bars = 20

        [input]
        method = pulse
        source = auto

        [output]
        method = raw
        raw_target = /dev/stdout
        data_format = ascii
        ascii_max_range = 7
    "#;

    let tmp_config = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(confin_path);

    tmp_config.unwrap().write_all(config.as_bytes()).unwrap();
}

fn main() -> io::Result<()> {
    let config_path = String::from("/tmp/cava_waybar_config");
    let playerctl_status = Arc::new(AtomicBool::from(false));

    let playerctl_trd = {
        let playerctl_status = Arc::clone(&playerctl_status);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            playerctl_active(&playerctl_status);
        })
    };

    init_cava_config(&config_path);
    let cava_trd = {
        let playerctl_status = Arc::clone(&playerctl_status);
        std::thread::spawn(move || {
            let _ = format_bar(&config_path, &playerctl_status);
        })
    };

    let _ = playerctl_trd.join();
    let _ = cava_trd.join();

    Ok(())
}
