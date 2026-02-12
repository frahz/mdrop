# mdrop

CLI and GUI tools for controlling Moondrop USB audio dongles on Linux and macOS.

## Supported devices

- Moondrop Dawn Pro
- Moondrop Dawn 3.5mm (not tested)
- Moondrop Dawn 4.4mm (not tested)

## Install

### Requirements

You need the following udev rule to communicate with the dongle:

```udev
SUBSYSTEM=="usb", ATTRS{idVendor}=="2fc6", MODE="0666"
```

### Nix

```sh
# cli
nix run github:frahz/mdrop

# gui
nix run github:frahz/mdrop#gui
```

## Usage

![image](https://github.com/user-attachments/assets/30fdb3ac-fd8a-440c-a7a0-d31f74788fda)

```sh
$ mdrop --help
A tool to control your Moondrop dongle

Usage: mdrop [OPTIONS] <COMMAND>

Commands:
  get      Gets status of Moondrop dongle
  set      Sets various values in your Moondrop dongle
  devices  Lists all the Moondrop dongles connected to the PC
  help     Print this message or the help of the given subcommand(s)

Options:
  -s <DEVICE>      Specify target device by USB bus and address, e.g. `03:02`
      --json       Print output as JSON
  -h, --help       Print help
```

### Device selection behavior

- No dongles connected: command fails with a clear error.
- One dongle connected: command auto-selects that dongle.
- Multiple dongles connected: command fails unless `-s` is provided.

### Examples

```sh
$ mdrop devices
┌───────name────────┬──bus──┬volume┬───────────filter───────────┬─gain─┬indicator_state┐
│ MOONDROP Dawn Pro │ 03:28 │ 81%  │ Fast roll-off, low-latency │ High │ Disabled      │
└───────────────────┴───────┴──────┴────────────────────────────┴──────┴───────────────┘

$ mdrop --json devices
[{"name":"MOONDROP Dawn Pro","bus":"03:28","volume":81,"filter":"Fast roll-off, low-latency","gain":"High","indicator_state":"Disabled"}]

$ mdrop --json get volume
{"value":81}

$ mdrop -s 03:28 get all
┌───────name────────┬──bus──┬volume┬───────────filter───────────┬─gain─┬indicator_state┐
│ MOONDROP Dawn Pro │ 03:28 │ 81%  │ Fast roll-off, low-latency │ High │ Disabled      │
└───────────────────┴───────┴──────┴────────────────────────────┴──────┴───────────────┘

$ mdrop set filter --help
Sets audio filter

Usage: mdrop set filter [OPTIONS] <FILTER>

Arguments:
  <FILTER>  [possible values: fast-roll-off-low-latency, fast-roll-off-phase-compensated, slow-roll-off-low-latency, slow-roll-off-phase-compensated, non-oversampling]
```


