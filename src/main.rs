use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::select;

use std::process::{self as proc};
pub mod cli;
pub mod db;
pub mod debug;

use clap::Parser as _;
use json;

async fn run_command(cmd: String) -> Result<proc::ExitStatus, io::Error> {
    let (prog, arg1) = match cfg!(target_os = "windows") {
        true => ("cmd", "/C"),
        false => ("sh", "-c"),
    };

    let mut app = Command::new(prog)
        .args([arg1, &cmd])
        .stdin(proc::Stdio::piped())
        .stdout(proc::Stdio::piped())
        .spawn()?;

    let mut app_stdin = match app.stdin.take() {
        Some(stdin) => stdin,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find application stdin",
            ))
        }
    };
    let app_stdout = match app.stdout.take() {
        Some(stdout) => stdout,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find application stdout",
            ))
        }
    };

    let mut reader = BufReader::new(app_stdout);
    let mut stdin_reader = BufReader::new(tokio::io::stdin());
    let mut input = String::new();

    loop {
        let mut buffer = [0; 1]; // Buffer to hold one byte

        select! {
            Ok(n) = reader.read(&mut buffer) => {
                if n == 0 {
                    break;
                }
                print!("{}", buffer[0] as char);
                io::stdout().flush().await.unwrap();  // Make sure the byte is printed immediately
            }
            Ok(n) = stdin_reader.read_line(&mut input) => {
                if n > 0 {
                    app_stdin.write_all(input.as_bytes()).await?;
                    app_stdin.flush().await?;
                    input.clear();
                }
            }
        }
    }

    return app.wait().await;
}

#[tokio::main]
async fn main() {
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
            proc::exit(err.exit_code());
        }
    };

    match args.command {
        cli::Command::Call { cmd, args } => {
            let cmd_args: String = args.join(" ");
            let cmd_path = &map["cmds"][&cmd];

            if cmd_path.is_null() {
                debug::print(format!("Command not found in DB: {}", &cmd));
            } else {
                match run_command(format!("{} {}", cmd_path, cmd_args)).await {
                    Ok(_status) => {}
                    Err(err) => debug::print(format!("Error running command: {}", err)),
                }
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
