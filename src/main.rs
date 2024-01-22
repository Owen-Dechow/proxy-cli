use std::process::{self as proc, exit};

pub mod cli;
pub mod db;
pub mod debug;

use clap::Parser as _;
use json;

fn run_command(cmd: String) {
    let result = proc::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(proc::Stdio::piped())
        .output();

    match result {
        Ok(out) => {
            let std_out = String::from_utf8_lossy(&out.stdout);
            let std_err = String::from_utf8_lossy(&out.stderr);

            if std_out.len() > 0 {
                println!("\n{}", std_out);
            }

            if std_err.len() > 0 {
                println!("\n{}", std_err);
            }
        }
        Err(err) => debug::print(err.to_string()),
    };
}

fn main() {
    let mut map = match db::get_db_content() {
        Ok(map) => map,
        Err(err) => match err {
            db::DBError::IOError(err) => panic!("{}", err),
            db::DBError::JsonError(err) => panic!("{}", err),
        },
    };

    let args = match cli::Arguments::try_parse() {
        Ok(args) => args,
        Err(err) => {
            debug::print(format!("{}", err));
            exit(err.exit_code());
        }
    };

    match args.command {
        cli::Command::Call { cmd, args } => {
            let cmd_args: String = args.join(" ");
            let cmd_path = &map["cmds"][&cmd];

            if cmd_path.is_null() {
                debug::print(format!("Command not found in DB: {}", &cmd));
            } else {
                run_command(format!("{} {}", cmd_path, cmd_args));
            }
        }
        cli::Command::Add { path } => {
            let split_key = path
                .components()
                .last()
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap()
                .split(".")
                .collect::<Vec<&str>>();

            let key = match split_key.len() {
                1 => split_key[0].to_string(),
                _ => split_key[..split_key.len() - 1].join("."),
            };

            let value = json::JsonValue::String(path.to_str().unwrap().to_string());
            match map["cmds"].insert(key.as_str(), value) {
                Ok(_) => {
                    json::JsonValue::String(path.to_str().unwrap().to_string());
                    db::attempt_db_save(
                        &map.dump(),
                        format!("Application successfully added to db under proxy: {}", key),
                    );
                }
                Err(err) => debug::print(format!("Could not insert command: {}", err)),
            }
        }
        cli::Command::Remove { cmd } => {
            let key = cmd.as_str();
            map["cmds"].remove(key);
            db::attempt_db_save(
                &map.dump(),
                format!("Command successfully removed from db: {}", cmd),
            );
        }
        cli::Command::List => {
            let mut list = String::new();
            for (key, val) in map["cmds"].entries() {
                list.push_str(format!("{}: {}\n", key, val).as_str())
            }
            if list.len() > 0 {
                debug::print(list);
            } else {
                debug::print("No proxies registered".to_string())
            }
        }
    };
}
