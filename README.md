# Mac sleep http api

An api that can put Mac's screen to sleep or wake up the display.  
With status check support and Home Assistant-compatible REST endpoints.

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

check <http://localhost:17698/status> to see if it works

## API endpoints

```text
POST /on
POST /off
GET  /status

GET  /ha/sensor
GET  /ha/binary_sensor
GET  /ha/binary_sensor/json
GET  /ha/switch
POST /ha/switch
PUT  /ha/switch
PATCH /ha/switch
```

### Home Assistant payloads

`GET /ha/sensor` returns JSON:

```json
{
  "state": "on",
  "switch_state": "ON",
  "is_on": true
}
```

`GET /ha/binary_sensor` returns plain text `on` or `off`.

`GET /ha/switch` returns plain text `ON` or `OFF`.

`POST/PUT/PATCH /ha/switch` accepts:

- plain text: `ON`, `OFF`, `true`, `false`, `1`, `0`
- JSON: `{"state":"ON"}` / `{"command":"off"}` / `{"active":true}` / `{"is_on":false}`

The response body is the current switch state (`ON` or `OFF`).

## install as service

```bash
mac-sleep-api install
```

logout and login again  
See if it's working by going to <http://localhost:17698/status>

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

## Home Assistant examples

```yaml
sensor:
  - platform: rest
    name: "Mac Display State"
    resource: "http://MAC_IP:17698/ha/sensor"
    value_template: "{{ value_json.state }}"
```

```yaml
binary_sensor:
  - platform: rest
    name: "Mac Display On"
    resource: "http://MAC_IP:17698/ha/binary_sensor"
    device_class: power
```

```yaml
switch:
  - platform: rest
    name: "Mac Display"
    resource: "http://MAC_IP:17698/ha/switch"
    method: post
    body_on: "ON"
    body_off: "OFF"
```
