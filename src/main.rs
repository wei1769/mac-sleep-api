#[macro_use]
extern crate rocket;
use clap::{Parser, Subcommand};
use process_path::get_executable_path;
use rocket::log::LogLevel;
use rocket::response::content;
use rocket::{get, http::Status};
use std::fs;
use std::process::{self, Command};
use std::str::{self};
mod cors;
use rocket::config::Config;
use std::str::FromStr;
use whoami::username;

#[derive(Parser, Debug)]
pub struct Arguments {
    #[arg(short, long, value_name = "address", default_value = "0.0.0.0")]
    bind_address: String,
    #[arg(short, long, value_name = "port", default_value_t = 17698)]
    port: u16,
    #[arg(short, long)]
    verbose: bool,
    #[command(subcommand)]
    install: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Start,
    Install,
}
#[launch]
async fn rocket() -> _ {
    let args = Arguments::parse();
    let address =
        std::net::IpAddr::from_str(args.bind_address.as_str()).expect("bind address invalid");
    let port = args.port;
    let username = username();
    match args.install {
        Commands::Start => {
            let config = Config {
                port,
                address,
                workers: 1,
                log_level: if args.verbose {
                    LogLevel::Off
                } else {
                    LogLevel::Critical
                },
                cli_colors: true,
                ..Config::release_default()
            };
            rocket::custom(config)
                .attach(cors::stage())
                .mount(
                    "/",
                    routes![all_options, get_status, set_display_off, set_display_on],
                )
                .register("/", catchers![general_not_found])
        }
        Commands::Install => {
            let path = match get_executable_path() {
                Some(s) => s.to_str().unwrap().to_string(),
                None => format!("/Users/{username}/.cargo/bin/mac-sleep-api",),
            };
            fs::create_dir_all(format!("/Users/{username}/.msa",)).expect("error creating ~/.msa");
            let plist = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
            <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
            <plist version="1.0">
            <dict>
                    <key>Label</key>
                    <string>mac.sleep.api</string>
                    <key>UserName</key>
                    <string>{username}</string>
                    <key>ProgramArguments</key>
                    <array>
                            <string>{path}</string>
                            <string>-v</string>
                            <string>-b</string>
                            <string>{address}</string>
                            <string>-p</string>
                            <string>{port}</string>
                            <string>start</string>
                    </array>
                    <key>WorkingDirectory</key>
                    <string>/Users/{username}/.msa</string>
                    <key>StandardOutPath</key>
                    <string>/Users/{username}/.msa/log</string>
                    <key>StandardErrorPath</key>
                    <string>/Users/{username}/.msa/error</string>
                    <key>KeepAlive</key>
                    <true/>
            </dict>
            </plist>
            "#,
            );
            let file = fs::write(
                format!("/Users/{username}/Library/LaunchAgents/mac.sleep.api.plist"),
                plist,
            )
            .unwrap();
            process::exit(0)
        }
    }
}

#[options("/<_..>")]
pub fn all_options() -> Result<(), ()> {
    Ok(())
}
#[post("/on")]
pub fn set_display_on() -> Result<(), Status> {
    let output = Command::new("caffeinate").args(["-u", "-t", "1"]).output();
    match output {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{:?}", e);
            Err(Status { code: 500 })
        }
    }
}
#[post("/off")]
pub fn set_display_off() -> Result<(), Status> {
    let output = Command::new("pmset").args(["displaysleepnow"]).output();
    match output {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{:?}", e);
            Err(Status { code: 500 })
        }
    }
}

#[get("/status")]
pub fn get_status() -> Result<String, Status> {
    let output = Command::new("system_profiler")
        .args(["SPDisplaysDataType"])
        .output();
    match output {
        Ok(s) => {
            let stdout = str::from_utf8(&s.stdout).unwrap();
            if stdout.to_string().contains("Asleep: Yes") {
                Ok("off".to_string())
            } else {
                Ok("on".to_string())
            }
        }
        Err(e) => {
            println!("{:?}", e);
            Err(Status { code: 500 })
        }
    }
}

#[catch(404)]
pub fn general_not_found() -> content::RawHtml<String> {
    content::RawHtml("404".to_string())
}
