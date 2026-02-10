# mdrop CLI tool

Linux/MacOS CLI tool for controlling Moondrop USB audio dongles.

## Usage

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
  -s <DEVICE>      specify target device, by using the USB bus number, to which the command should be directed, ex. `03:02`
      --json       Print output as JSON
  -h, --help       Print help
```
### Example

```sh
$ mdrop devices
┌───────name────────┬──bus──┬volume┬───────────filter───────────┬─gain─┬indicator_state┐
│ MOONDROP Dawn Pro │ 03:28 │ 81%  │ Fast roll-off, low-latency │ High │ Disabled      │
└───────────────────┴───────┴──────┴────────────────────────────┴──────┴───────────────┘

$ mdrop --json devices
[{"name":"MOONDROP Dawn Pro","bus":"03:28","volume":81,"filter":"Fast roll-off, low-latency","gain":"High","indicator_state":"Disabled"}]

$ mdrop --json get volume
{"value":81}
```

![image](https://github.com/user-attachments/assets/30fdb3ac-fd8a-440c-a7a0-d31f74788fda)


## Supported devices

- Moondrop Dawn Pro
- Moondrop Dawn 3.5mm (not tested)
- Moondrop Dawn 4.4mm (not tested)

## Install

### Requirements

You will need the following udev rules to be able to communicate with the dongle:

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

## TODO List

- Add option to specify device to configure using `bus` and `address` number (because Moondrop doesn't give unique serial ids to the dongles)
- change the code to only support single device (most people won't have two Moondrop devices connected at the same time)
