# cargo-vsc • autogenerate .vscode folders

[![GitHub](https://img.shields.io/github/stars/MaulingMonkey/cargo-vsc.svg?label=GitHub&style=social)](https://github.com/MaulingMonkey/cargo-vsc)
[![crates.io](https://img.shields.io/crates/v/cargo-vsc.svg)](https://crates.io/crates/cargo-vsc)
[![%23![forbid(unsafe_code)]](https://img.shields.io/github/search/MaulingMonkey/cargo-vsc/unsafe%2bextension%3Ars?color=green&label=%23![forbid(unsafe_code)])](https://github.com/MaulingMonkey/cargo-vsc/search?q=forbid%28unsafe_code%29+extension%3Ars)
[![rust: stable](https://img.shields.io/badge/rust-stable-yellow.svg)](https://gist.github.com/MaulingMonkey/c81a9f18811079f19326dac4daa5a359#minimum-supported-rust-versions-msrv)
[![License](https://img.shields.io/crates/l/cargo_vsc.svg)](https://github.com/MaulingMonkey/cargo-vsc)
[![Build Status](https://travis-ci.com/MaulingMonkey/cargo-vsc.svg?branch=master)](https://travis-ci.com/MaulingMonkey/cargo-vsc)



<h2 name="quickstart">Quickstart</h2>

```sh
cd my-rust-project
cargo install cargo-vsc
cargo vsc
code .
```

`Ctrl` + `Shift` + `B` to check, test, and build<br>
`Ctrl` + `Shift` + `D` "Run" to select different executables<br>
`F5` to debug the selected launch configuration (`cargo-vsc • debug` by default for this project)<br>



<h2 name="generated">What's generated?</h2>

`.vscode/.gitignore` since many/most projects don't want .vscode boilerplate checked in IME (although I always provide mine)<br>
`.vscode/extensions.json` so VS Code will auto-recommend appropriate extensions<br>
`.vscode/settings.json` to ignore `target` mucking up search results<br>
`.vscode/tasks.json` to check/build/test by default build action, open various documentation links as vanilla tasks, and to support launch.json<br>
`.vscode/launch.json` to provide debugging configurations for every rust bin and example in the workspace<br>



<h2 name="license">License</h2>

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.



<h2 name="contribution">Contribution</h2>

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
