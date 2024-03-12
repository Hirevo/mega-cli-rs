Changelog
=========

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

[Unreleased]
------------

### Added

### Changed

### Fixed

### Removed

[0.1.1] - 2024-03-12
--------------------

### Added

- Added `compare` subcommand
- Added `--origin` global command-line option
- Added `--max-retries` global command-line option
- Added `--min-retry-delay` global command-line option
- Added `--max-retry-delay` global command-line option
- Added `--timeout` global command-line option
- Added `--https` global command-line option
- Added `-H` shorthand for `--show-handles` in `list` and `tree` subcommands
- Added `--all` option for `get` subcommand
- `get` subcommand now uses checksums to avoid uselessely re-downloading files

### Changed

- `get` subcommand now chooses default output paths more sensibly

### Fixed

- Download errors in `get` subcommand are no longer silent

[0.1.0] - 2023-06-17
--------------------

### Added

- Initial release
