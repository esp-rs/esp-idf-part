# esp-idf-part

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/jessebraham/esp-idf-part/CI?label=CI&logo=github&style=flat-square)
![MSRV](https://img.shields.io/badge/MSRV-1.60-blue?style=flat-square)

A library for parsing and generating ESP-IDF partition tables. Supports parsing from and generating to both CSV and Binary formats.

> **Warning**
>
> This crate is still in early development and may not handle all edge cases. Use at your own risk and, please, report any issues that you may find!

This package started its life as the implementation in [espflash](https://github.com/esp-rs/espflash/), however it has undergone some fairly extensive changes since being extracted into its own crate.

## Resources

- [ESP-IDF Partition Table API Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
