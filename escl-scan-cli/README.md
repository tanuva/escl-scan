# escl-scan-cli

[![github actions](https://github.com/tanuva/escl-scan/workflows/Rust/badge.svg)](https://github.com/tanuva/escl-scan/actions)
[![dependency status](https://deps.rs/repo/github/tanuva/escl-scan/status.svg)](https://deps.rs/repo/github/tanuva/escl-scan)

---

```
Scans documents from an eSCL-speaking scanner on the network. The first
discovered scanner will be used if neither --name nor --host are provided.

Usage: escl-scan-cli [OPTIONS] [OUTPUT_FILE_NAME]

Arguments:
  [OUTPUT_FILE_NAME]  Output file name [default: scan.jpg]

Options:
      --host <HOST>                    Select scanner by IP or hostname
  -n, --name <NAME>                    Select scanner by device name (can be partial)
  -l, --list                           List available scanners
  -s, --source <INPUT_SOURCE>          Document source [default: platen] [possible values: camera, feeder, platen]
  -i, --input-format <INPUT_FORMAT>    Input document format [default: a4-portrait] [possible values: a4-portrait, a5-landscape, a5-portrait, us-letter-portrait]
  -r, --resolution <DPI>               Scan resolution in DPI (Dots Per Inch) [default: 300]
  -b, --base-path <OUTPUT_BASE_PATH>   Base path; will be prepended to the given output file name
  -o, --output-format <OUTPUT_FORMAT>  Output document format [default: jpg] [possible values: jpg, pdf]
  -c, --color <COLOR>                  Color mode [default: rgb] [possible values: black-and-white, grayscale, rgb]
  -h, --help                           Print help
  -V, --version                        Print version

```
