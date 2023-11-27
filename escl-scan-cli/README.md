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
      --host <HOST>       Select scanner by IP or hostname
  -n, --name <NAME>       Select scanner by device name (can be partial)
  -i, --info <INFO>       Print information about the scanner identified by device name
  -l, --list              List available scanners
  -f, --force             Overwrite the output file if it already exists
  -r, --resolution <DPI>  Scan resolution in DPI (Dots Per Inch) [default: 75]
  -h, --help              Print help
  -V, --version           Print version
```
