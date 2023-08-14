# Mac sleep http api

An api that can put Mac's screen to sleep or wake up the display.  
With status check support

## install

install cargo, build binary from source

```bash
# install cargo
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# press enter to proceed
$ source "$HOME/.cargo/env"
$ cargo install --git https://github.com/wei1769/mac-sleep-api.git
```

## testing

```bash
$ mac-sleep-api start
🚀 Rocket has launched from http://0.0.0.0:17698
```

check http://localhost:17698/status to see if it works

## install as service

```bash
mac-sleep-api install
```

logout and login again  
See if it's working by going to http://localhost:17698/status

## cli options

```bash
  -b, --bind-address <address>  [default: 0.0.0.0]
  -p, --port <port>             [default: 17698]
  -v, --verbose
```

## sleep, awake with curl

```bash
# turning on
$ curl -X POST  localhost:17698/on
# turning off
$ curl -X POST  localhost:17698/off
```
