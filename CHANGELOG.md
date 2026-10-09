# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-09

First GitHub release of the Rust port and the Godot chart.

### Added

- The simulation crate, checked against the Python game at `9b02494`.
- A command-line runner: `new`, `script`, and `load`.
- The Godot 4.7 chart: new game, save and load, sail, market, contracts, crew, shipyard, harbour, hunt, journal, the day's report, and sea fights.
- The landing bundle at manifest 0.4.3. The pictures are not part of the MIT grant.
- A landing page and a handbook.
- Patch coverage fails under 90%. The repository-wide percentage aims at 90% and stays informational. The Godot crate is outside that report.

### Changed

- `portlight --help` lists the script commands the runner actually accepts, and the exit codes it uses.

## [Unreleased]
