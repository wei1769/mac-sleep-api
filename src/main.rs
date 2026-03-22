#[macro_use]
extern crate rocket;

use clap::{Parser, Subcommand};
use rocket::config::Config;
use rocket::http::Status;
use rocket::log::LogLevel;
use rocket::response::content;
use rocket::serde::json::{serde_json, Json, Value};
use rocket::serde::Serialize;
use std::fs;
use std::process::{self, Command};
use std::str::FromStr;
use whoami::username;
mod cors;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisplayState {
    On,
    Off,
}

impl DisplayState {
    fn status_value(self) -> &'static str {
        match self {
            DisplayState::On => "on",
            DisplayState::Off => "off",
        }
    }

    fn switch_value(self) -> &'static str {
        match self {
            DisplayState::On => "ON",
            DisplayState::Off => "OFF",
        }
    }

    fn is_on(self) -> bool {
        matches!(self, DisplayState::On)
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct HomeAssistantStatePayload {
    state: String,
    switch_state: String,
    is_on: bool,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct HomeAssistantBinaryPayload {
    state: String,
    open: bool,
}

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
                    routes![
                        all_options,
                        get_status,
                        set_display_off,
                        set_display_on,
                        get_ha_sensor,
                        get_ha_binary_sensor,
                        get_ha_binary_sensor_json,
                        get_ha_switch,
                        post_ha_switch,
                        put_ha_switch,
                        patch_ha_switch
                    ],
                )
                .register("/", catchers![general_not_found])
        }
        Commands::Install => {
            let path = std::env::current_exe()
                .ok()
                .and_then(|value| value.into_os_string().into_string().ok())
                .unwrap_or_else(|| format!("/Users/{username}/.cargo/bin/mac-sleep-api"));
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
            fs::write(
                format!("/Users/{username}/Library/LaunchAgents/mac.sleep.api.plist"),
                plist,
            )
            .expect("error adding to launch item");
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
    set_display_state(DisplayState::On)
}

#[post("/off")]
pub fn set_display_off() -> Result<(), Status> {
    set_display_state(DisplayState::Off)
}

#[get("/status")]
pub fn get_status() -> Result<String, Status> {
    Ok(get_display_state()?.status_value().to_string())
}

#[get("/ha/sensor")]
pub fn get_ha_sensor() -> Result<Json<HomeAssistantStatePayload>, Status> {
    let display_state = get_display_state()?;
    Ok(Json(build_ha_state_payload(display_state)))
}

#[get("/ha/binary_sensor")]
pub fn get_ha_binary_sensor() -> Result<String, Status> {
    let display_state = get_display_state()?;
    Ok(display_state.status_value().to_string())
}

#[get("/ha/binary_sensor/json")]
pub fn get_ha_binary_sensor_json() -> Result<Json<HomeAssistantBinaryPayload>, Status> {
    let display_state = get_display_state()?;
    Ok(Json(HomeAssistantBinaryPayload {
        state: display_state.status_value().to_string(),
        open: display_state.is_on(),
    }))
}

#[get("/ha/switch")]
pub fn get_ha_switch() -> Result<String, Status> {
    let display_state = get_display_state()?;
    Ok(display_state.switch_value().to_string())
}

#[post("/ha/switch", data = "<payload>")]
pub fn post_ha_switch(payload: String) -> Result<String, Status> {
    set_ha_switch(payload)
}

#[put("/ha/switch", data = "<payload>")]
pub fn put_ha_switch(payload: String) -> Result<String, Status> {
    set_ha_switch(payload)
}

#[patch("/ha/switch", data = "<payload>")]
pub fn patch_ha_switch(payload: String) -> Result<String, Status> {
    set_ha_switch(payload)
}

fn set_ha_switch(payload: String) -> Result<String, Status> {
    let Some(target_state) = parse_switch_payload(&payload) else {
        eprintln!("Unsupported /ha/switch payload: {:?}", payload);
        return Err(Status::BadRequest);
    };
    set_display_state(target_state)?;
    Ok(get_display_state()?.switch_value().to_string())
}

fn parse_switch_payload(payload: &str) -> Option<DisplayState> {
    let trimmed = payload.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(parsed) = parse_text_state(trimmed) {
        return Some(parsed);
    }

    let json = serde_json::from_str::<Value>(trimmed).ok()?;
    extract_state_from_json(&json)
}

fn extract_state_from_json(value: &Value) -> Option<DisplayState> {
    for key in ["state", "command", "active", "is_active", "is_on"] {
        if let Some(candidate) = value.get(key) {
            if let Some(parsed) = parse_json_state(candidate) {
                return Some(parsed);
            }
        }
    }

    None
}

fn parse_json_state(value: &Value) -> Option<DisplayState> {
    match value {
        Value::Bool(flag) => Some(if *flag {
            DisplayState::On
        } else {
            DisplayState::Off
        }),
        Value::Number(number) => match number.as_i64() {
            Some(1) => Some(DisplayState::On),
            Some(0) => Some(DisplayState::Off),
            _ => None,
        },
        Value::String(raw) => parse_text_state(raw),
        _ => None,
    }
}

fn parse_text_state(raw: &str) -> Option<DisplayState> {
    let lowered = raw.trim().trim_matches('"').to_ascii_lowercase();
    match lowered.as_str() {
        "on" | "true" | "1" | "open" => Some(DisplayState::On),
        "off" | "false" | "0" | "closed" => Some(DisplayState::Off),
        _ => None,
    }
}

fn build_ha_state_payload(display_state: DisplayState) -> HomeAssistantStatePayload {
    HomeAssistantStatePayload {
        state: display_state.status_value().to_string(),
        switch_state: display_state.switch_value().to_string(),
        is_on: display_state.is_on(),
    }
}

fn get_display_state() -> Result<DisplayState, Status> {
    let output = Command::new("system_profiler")
        .args(["SPDisplaysDataType"])
        .output();
    match output {
        Ok(s) if s.status.success() => {
            let stdout = std::str::from_utf8(&s.stdout).map_err(|e| {
                eprintln!("Invalid UTF-8 output from system_profiler: {:?}", e);
                Status::InternalServerError
            })?;

            if stdout.contains("Asleep: Yes") {
                Ok(DisplayState::Off)
            } else {
                Ok(DisplayState::On)
            }
        }
        Ok(s) => {
            eprintln!(
                "system_profiler exited with {} and stderr: {}",
                s.status,
                String::from_utf8_lossy(&s.stderr)
            );
            Err(Status::InternalServerError)
        }
        Err(e) => {
            eprintln!("Failed to call system_profiler: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

fn set_display_state(state: DisplayState) -> Result<(), Status> {
    let command_output = match state {
        DisplayState::On => Command::new("caffeinate").args(["-u", "-t", "1"]).output(),
        DisplayState::Off => Command::new("pmset").args(["displaysleepnow"]).output(),
    };

    match command_output {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            eprintln!(
                "Display command exited with {} and stderr: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
            Err(Status::InternalServerError)
        }
        Err(e) => {
            eprintln!("Failed to execute display command: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[catch(404)]
pub fn general_not_found() -> content::RawHtml<String> {
    content::RawHtml("404".to_string())
}
