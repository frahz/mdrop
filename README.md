# mdrop CLI tool

Linux CLI tool for controlling Moondrop USB audio dongles.

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
  -h, --help       Print help
```

## Supported devices

- Moondrop Dawn Pro
- Moondrop Dawn 3.5mm (not tested)
- Moondrop Dawn 4.4mm (not tested)

## TODO List

- Add option to specify device to configure using `bus` and `address` number (because Moondrop doesn't give unique serial ids to the dongles)
- Make some command output look nicer
- Better error handling instead of `unwrap`ing everything.
