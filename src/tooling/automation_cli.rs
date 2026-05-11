use std::fs;
use std::thread;
use std::time::Duration;

use crate::automation;

pub const AUTOMATION_DIR: &str = ".zig-cache/zero-native-automation";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    UnsupportedCommand,
    AutomationFailed,
    Timeout,
}

pub fn run(args: &[String]) -> Result<(), Error> {
    if args.is_empty() {
        print_usage();
        return Ok(());
    }
    match args[0].as_str() {
        "list" => print_file("windows.txt"),
        "snapshot" => print_file("snapshot.txt"),
        "screenshot" => {
            eprintln!("screenshot capture is not available for this backend");
            return Err(Error::UnsupportedCommand);
        }
        "reload" => send_command("reload", ""),
        "wait" => wait_for_file("snapshot.txt", "ready=true"),
        "bridge" => {
            if args.len() < 2 {
                print_usage();
                return Ok(());
            }
            let _ = delete_automation_file("bridge-response.txt");
            send_command("bridge", &args[1])?;
            wait_for_file("bridge-response.txt", "")?;
            Ok(())
        }
        _ => {
            print_usage();
            Ok(())
        }
    }
}

fn print_file(name: &str) -> Result<(), Error> {
    let path = format!("{}/{}", AUTOMATION_DIR, name);
    match fs::read_to_string(&path) {
        Ok(content) => {
            print!("{}", content);
            Ok(())
        }
        Err(_) => {
            eprintln!("error: no app connected");
            Err(Error::AutomationFailed)
        }
    }
}

fn send_command(action: &str, value: &str) -> Result<(), Error> {
    let _ = fs::create_dir_all(AUTOMATION_DIR);
    let _cmd = automation::Command {
        action: match action {
            "reload" => automation::Action::Reload,
            "wait" => automation::Action::Wait,
            "bridge" => automation::Action::Bridge,
            _ => return Err(Error::UnsupportedCommand),
        },
        value: value.to_string(),
    };
    let line = format!("{} {}", action, value).trim().to_string();
    let command_path = format!("{}/command.txt", AUTOMATION_DIR);
    fs::write(&command_path, format!("{}\n", line)).map_err(|_| Error::AutomationFailed)?;
    println!("queued {}", action);
    Ok(())
}

fn wait_for_file(name: &str, marker: &str) -> Result<(), Error> {
    let path = format!("{}/{}", AUTOMATION_DIR, name);
    let mut attempts = 0;
    while attempts < 50 {
        match fs::read_to_string(&path) {
            Ok(content) => {
                if marker.is_empty() || content.contains(marker) {
                    print!("{}", content);
                    return Ok(());
                }
            }
            Err(_) => {}
        }
        thread::sleep(Duration::from_millis(100));
        attempts += 1;
    }
    eprintln!("error: timed out waiting for automation");
    Err(Error::Timeout)
}

fn delete_automation_file(name: &str) -> Result<(), Error> {
    let path = format!("{}/{}", AUTOMATION_DIR, name);
    let _ = fs::remove_file(&path);
    Ok(())
}

fn print_usage() {
    eprintln!(
        "usage: zero-native automate <command>

commands:
  list
  snapshot
  screenshot
  reload
  wait
  bridge <request-json>"
    );
}
