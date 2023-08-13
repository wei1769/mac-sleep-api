#[macro_use]
extern crate rocket;
use clap::Parser;
use rocket::response::content;
use rocket::{get, http::Status};
use std::process::Command;
use std::str::{self};
mod cors;
use rocket::config::Config;
use std::str::FromStr;

#[derive(Parser, Default, Debug)]
pub struct Arguments {
    #[arg(short, long, value_name = "address")]
    bind_address: Option<String>,
    #[arg(short, long, value_name = "port")]
    port: Option<u16>,
}

#[launch]
async fn rocket() -> _ {
    let args = Arguments::parse();
    let addr = match args.bind_address {
        Some(s) => std::net::IpAddr::from_str(&s),
        None => std::net::IpAddr::from_str("0.0.0.0"),
    }
    .expect("bind address invalid");

    let config = Config {
        port: args.port.unwrap_or(17698),
        address: addr,
        workers: 1,
        log_level: rocket::log::LogLevel::Critical,
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
