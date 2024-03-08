<div align=center><h1>mega-cli-rs</h1></div>
<div align=center><strong>A command-line tool for interacting with MEGA</strong></div>

<br />

<div align="center">
  <!-- crate version -->
  <a href="https://crates.io/crates/mega-cli">
    <img src="https://img.shields.io/crates/v/mega-cli" alt="crates.io version" />
  </a>
  <!-- crate downloads -->
  <a href="https://crates.io/crates/mega-cli">
    <img src="https://img.shields.io/crates/d/mega-cli" alt="crates.io download count" />
  </a>
  <!-- crate license -->
  <a href="https://github.com/Hirevo/mega-rs#license">
    <img src="https://img.shields.io/crates/l/mega-cli" alt="crate license" />
  </a>
</div>

About
-----

`mega-cli-rs` (`mega-cli` on [crates.io] and once installed) is an (unofficial) command-line tool for interacting with [MEGA][mega.nz].  

It aims to implement a lot (if not all) of the features offered by [MEGAcmd] or [megatools].  

[crates.io]: https://crates.io/crates/mega-cli
[mega.nz]: https://mega.nz
[MEGAcmd]: https://github.com/meganz/MEGAcmd
[megatools]: https://megatools.megous.com

It is written in Rust and uses the [mega][mega-rs] crate for its MEGA interactions.  

It serves as a real-life test for the [mega][mega-rs] crate, to assess metrics like performance, API ergonomics and feature-completeness.  

It can also serve as a large-scale example of how to use and get the most out of the library.

[mega-rs]: https://github.com/Hirevo/mega-rs

Installation
------------

You can use Cargo to install `mega-cli` by running the following command:

```bash
cargo install mega-cli
```

Supported Commands
------------------

- [x] `auth`: Manage authentication with MEGA
  - [x] `login`: Create a new persisted session with MEGA
  - [x] `logout`: Log out of the current session with MEGA
  - [x] `me`: Display information about the current session
- [x] `config`: Interact with the `mega-cli` configuration
  - [x] `path`: Display the path to the configuration file
  - [x] `edit`: Edit the configuration file with a text editor
- [x] `get`: Download owned files from MEGA
    - [x] Single file downloads
    - [x] Recursive folder downloads
    - [x] Parallel file downloads (during recursive folder downloads)
    - [x] Supports public and password-protected links (using `-l|--link` and `-p|--password`).
- [x] `put`: Upload files to MEGA
    - [x] Single file uploads
    - [ ] Recursive folder uploads
    - [ ] Parallel file uploads (during recursive folder uploads)
- [x] `list`: List remote MEGA nodes
  - [x] Supports public and password-protected links (using `-l|--link` and `-p|--password`).
- [x] `tree`: Display remote MEGA nodes recursively as a tree
  - [x] Supports public and password-protected links (using `-l|--link` and `-p|--password`).
- [x] `mkdir`: Create folders within MEGA
- [x] `rename`: Rename nodes within MEGA
- [x] `delete`: Delete remote MEGA nodes
- [x] `follow`: Display MEGA storage events as they happen

License
-------

Licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
