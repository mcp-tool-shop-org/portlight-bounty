# Windows and macOS build check

Investigation only. This note does not merge, does not change `main`, and does not set a Cargo target, a `rust-toolchain.toml` channel, or an `export_presets.cfg`.

## Commits

| Run | Commit | What it is |
| --- | --- | --- |
| Linux commands in this note | `587fe7834c37c3f3579a48516ab95a485dd35c50` | Head of PR #2 when this agent ran them. `godot/` tree `55a2187a3772ad6a24a9ab88177ddc480d9f6a2c`. `crates/portlight-godot` tree `ee06f89134bd3c0c93166af4150287c4be675390`. |
| Windows manual run | `110469f` | `main` after PR #2 merged. Run on Mike's Windows machine by another agent. Not this agent's CI. |
| Godot sources on main | `1b00b8f` | Reported identical to the Godot sources this agent tested. This agent did not re-diff that commit. |

This agent could not fetch `110469f` or `1b00b8f`. `git fetch origin main` failed with `Invalid username or token` (the VM credential had expired). Local `origin/main` is still `b2e4aba` (`Port fleet, injuries, and weapons (#8)`). Rebase onto current `main` was not easy, and it was not done. The branch is still the note commits on top of `587fe78`.

## How to read the table

**PASS**, **FAIL**, or **UNTESTED**. UNTESTED means that command was not run on that platform. A Linux cross-link is not a Windows result. Headless Godot is the editor binary with `--headless`. It uses the same GDExtension loader as the editor window. Nobody opened a clicked editor window.

The Windows column is Mike's machine, credited below. It is not a GitHub Actions result.

| Platform | Sim build | gdext build | Library load | Export |
| --- | --- | --- | --- | --- |
| Linux x86_64 | PASS | FAIL on Rust 1.83.0; PASS on Rust 1.98.1 | PASS (headless editor, debug `.so`) | FAIL as committed (no presets). PASS with a temporary Linux preset that was not committed |
| Windows x86_64 | PASS (compiled as a dependency of the gdext build; the sim test suite was not run) | PASS (`x86_64-pc-windows-msvc`, Rust 1.98.1) | FAIL with only the release DLL. PASS after a debug build | UNTESTED (no `export_presets.cfg`; the release DLL was not loaded) |
| macOS | UNTESTED | UNTESTED | UNTESTED | UNTESTED |

## Toolchains

| Tool | Version |
| --- | --- |
| This agent's host | Linux x86_64, kernel 6.12.94+ |
| Pinned file, left unchanged | `rust-toolchain.toml` channel `1.83.0`. rustc 1.83.0 (90b35a623 2024-11-26), cargo 1.83.0 (5ffbef321 2024-10-29) |
| Stable used for gdext on Linux | rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), via `RUSTUP_TOOLCHAIN=stable` or `cargo +1.98.1` |
| Mike's Windows host | rustc 1.98.1 (48a229cea 2026-09-01), host `x86_64-pc-windows-msvc`. `rust-toolchain.toml` left at 1.83. Command was `cargo +1.98.1`. |
| MinGW cross linker (Linux only, not loaded) | x86_64-w64-mingw32-gcc (GCC) 13-win32, thread model win32 |
| MSVC cross (Linux only, not loaded) | cargo-xwin 0.23.1, Ubuntu clang 18.1.3 |
| Godot on Linux | 4.7.2.stable.official.ed1daf0bf |
| Godot on Mike's Windows machine | 4.7.stable.official.5b4e0cb0f (not 4.7.2) |

`godot` 0.5.5 publishes `edition = "2024"` and `rust-version = "1.94"`. This repo sets no `rust-version`. The only pin is `rust-toolchain.toml`.

## Linux commands (this agent, commit 587fe78)

### Sim, Rust 1.83.0 — PASS

```text
cargo test --locked --workspace --exclude portlight-godot
```

Exit 0. Suites: 36, 102, 6 (parity), 10 (save parity). Doc tests empty. `portlight-godot` was excluded because Cargo 1.83 cannot parse the `godot` crate.

The same command with `RUSTUP_TOOLCHAIN=stable` (rustc 1.98.1) also exited 0, same suite counts.

### gdext, Rust 1.83.0 — FAIL

```text
cargo build --locked -p portlight-godot
```

Exit 101:

```text
error: failed to parse manifest at `/usr/local/cargo/registry/src/index.crates.io-6f17d22bba15001f/godot-0.5.5/Cargo.toml`

Caused by:
  feature `edition2024` is required

  The package requires the Cargo feature called `edition2024`, but that feature is not stabilized in this version of Cargo (1.83.0 (5ffbef321 2024-10-29)).
```

### gdext, Rust 1.98.1 — PASS

```text
export RUSTUP_TOOLCHAIN=stable
cargo test --locked -p portlight-godot
cargo build --locked -p portlight-godot
```

`cargo test` exit 0, 7 tests. `cargo build` wrote `target/debug/libportlight_godot.so`.

### Headless load — PASS

```text
Godot_v4.7.2-stable_linux.x86_64 --headless --path godot --import --quit
Godot_v4.7.2-stable_linux.x86_64 --headless --path godot -- --smoke
```

Both exited 0. Smoke log:

```text
Initialize godot-rust (API v4.7.stable.official, runtime v4.7.2.stable.official, safeguards strict)
Godot Engine v4.7.2.stable.official.ed1daf0bf - https://godotengine.org

portlight smoke ok
```

The loaded file was `res://../target/debug/libportlight_godot.so`, which is outside the Godot project folder.

### Export of the committed tree — FAIL

```text
Godot_v4.7.2-stable_linux.x86_64 --headless --path godot --export-release "Linux" /tmp/portlight-export/portlight.x86_64
```

Exit 1:

```text
ERROR: This project doesn't have an `export_presets.cfg` file at its root.
Create an export preset from the "Project > Export" dialog and try again.
   at: _fs_changed (editor/editor_node.cpp:1417)
```

### Export with a temporary Linux preset — PASS

The preset was not committed. Name `Linux`, platform `Linux`, `binary_format/architecture="x86_64"`, `texture_format/s3tc_bptc=true`, `texture_format/etc2_astc=false`, `binary_format/embed_pck=false`. Official 4.7.2 Linux templates were installed under `~/.local/share/godot/export_templates/4.7.2.stable/`.

```text
Godot_v4.7.2-stable_linux.x86_64 --headless --path godot --export-debug "Linux" /tmp/portlight-export/portlight.x86_64
```

Exit 0. The export directory contained `portlight.x86_64`, `portlight.pck`, `portlight.sh`, and `libportlight_godot.so`. The `.so` matched `target/debug/libportlight_godot.so`.

Copied to an empty directory, with `target/debug/libportlight_godot.so` moved aside:

```text
PORTLIGHT_SMOKE=1 ./portlight.x86_64 --headless -- --smoke
```

Exit 0, `Initialize godot-rust`, `portlight smoke ok`.

Same binary with the copied `.so` deleted:

```text
ERROR: Can't open dynamic library, file not found: '../target/debug/libportlight_godot.so'.
   at: open_dynamic_library (drivers/unix/os_unix.cpp:1063)
ERROR: GDExtension dynamic library not found: 'res://portlight.gdextension'.
   at: open_library (core/extension/gdextension.cpp:810)
ERROR: Error loading extension: 'res://portlight.gdextension'.
   at: load_extensions (core/extension/gdextension_manager.cpp:333)
ERROR: Cannot get class 'PortlightGame'.
   at: _instantiate_internal (core/object/class_db.cpp:587)
WARNING: Node Game of type PortlightGame cannot be created. A placeholder will be created instead.
```

That process did not exit. The smoke quit lives in the extension. It was killed after about ten minutes.

So on Linux, `res://../target/...` survives export only because Godot 4.7.2 globalizes the path, copies the file next to the binary, and `OS_Unix::open_dynamic_library` then finds that copy by file name. The packed `.gdextension` still says `../target/debug/libportlight_godot.so`. Delete the copy and the load fails.

## Windows manual run (Mike's machine, not CI)

Source: another agent, on Mike's Windows machine, commit `110469f`. Godot `4.7.stable.official.5b4e0cb0f`. `rust-toolchain.toml` was left at 1.83. Host rustc 1.98.1 (48a229cea 2026-09-01), `x86_64-pc-windows-msvc`.

### gdext release — PASS

```text
cargo +1.98.1 build --release -p portlight-godot
```

1m19s. Produced `target/release/portlight_godot.dll`, 6,940,672 bytes. `portlight-sim` and `portlight-chart` compile as dependencies of that crate, so the sim **build** is PASS. The sim test suite was not run on Windows.

### Editor load with only the release DLL — FAIL

There is no `export_presets.cfg`, so the only Godot run was headless editor import. The editor selects `windows.debug.x86_64`, which is `res://../target/debug/portlight_godot.dll`. The release DLL is a different file. Import exited 0 and still failed:

```text
GDExtension dynamic library not found: res://portlight.gdextension
```

at `os_windows.cpp:483` (`ERR_FILE_NOT_FOUND`), then `Cannot get class PortlightGame`.

Exit 0 is not a pass. Godot's Windows loader (`os_windows.cpp` around that line) tries the path from the `.gdextension` file and, if that file is missing, the same file name next to the Godot executable. It does not search `target/release` when the key says `target/debug`.

### gdext debug, then editor load — PASS

A debug build produced `target/debug/portlight_godot.dll`, 7,920,128 bytes. The extension loaded:

```text
Initialize godot-rust (API v4.7.stable.official, runtime v4.7.stable.official, safeguards strict)
portlight smoke ok
```

Conclusions from that machine:

- `res://../target/...` works in the Windows editor.
- An editor run needs the debug library from a plain `cargo +1.98.1 build -p portlight-godot` (no `--release`).
- The release DLL is what a release export would load (`windows.release.x86_64`). It was not loaded. Export is UNTESTED.
- The toolchain that was chosen and tested is `x86_64-pc-windows-msvc`.

## Linux cross-builds (not a substitute for Mike's run)

Gnu linker: `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc`.

Sim, Rust 1.83.0, both targets, exit 0. Neither exe was run.

```text
rustup +1.83.0 target add x86_64-pc-windows-gnu x86_64-pc-windows-msvc
RUSTUP_TOOLCHAIN=1.83.0 cargo build --locked -p portlight-cli --target x86_64-pc-windows-gnu
RUSTUP_TOOLCHAIN=1.83.0 cargo xwin build --locked -p portlight-cli --target x86_64-pc-windows-msvc
```

gdext, Rust 1.98.1, both targets, exit 0. Neither DLL was loaded. Mike's later msvc load is the one that counts.

```text
RUSTUP_TOOLCHAIN=stable cargo build --locked -p portlight-godot --target x86_64-pc-windows-gnu
RUSTUP_TOOLCHAIN=stable cargo xwin build --locked -p portlight-godot --target x86_64-pc-windows-msvc
```

The gnu DLL (about 185 MB debug) imports `msvcrt.dll` and was linked with Ubuntu's **win32** thread model. Godot's engine docs ask for **posix** MinGW. The msvc cross DLL (about 7.6 MB debug) imports `VCRUNTIME140.dll` plus the Universal CRT. Mike's tested debug DLL was 7,920,128 bytes, built on Windows with the msvc host, not this cross DLL.

## Recommendations

### 1. `res://../target/...`

**Editor load works.** Linux headless Godot 4.7.2 loaded the debug `.so`. Mike's Windows editor loaded the debug DLL. The path is the godot-rust development layout. It is outside the project folder, and that is fine for the editor.

The editor always resolves the **debug** key. `windows.debug.x86_64` is `res://../target/debug/portlight_godot.dll`. A release-only tree fails the editor even though `target/release/portlight_godot.dll` exists. The command that matches the editor is:

```text
cargo +1.98.1 build -p portlight-godot
```

**Export of the committed tree fails** because there is no `export_presets.cfg`. That was run on Linux and it matches the Windows limit (Mike could not export).

**With a preset, Linux Godot 4.7.2 did export `res://../target/...`.** It copied the library next to the game and the exported binary loaded that copy. Windows and macOS export of this path are UNTESTED. Godot 4.7.2's Windows loader has the same file-name fallback (`platform/windows/os_windows.cpp`). The macOS exporter copies shared objects to `Contents/Frameworks/`, and `OS_MacOS::open_dynamic_library` looks there (`../Frameworks` from `Contents/MacOS`). That macOS path was read from the 4.7.2 sources and not run.

For a ship build, point the **release** lines at files inside the project and copy them:

```text
cargo +1.98.1 build --release -p portlight-godot
cp target/release/libportlight_godot.so godot/bin/
```

Same for `portlight_godot.dll` and `libportlight_godot.dylib`. `windows.release.x86_64`, `macos.release`, `macos.release.arm64`, and `linux.release.x86_64` become `res://bin/...`. Leave the debug lines on `res://../target/debug/...`. Do not commit the binaries. Debug and release currently share a file name (`portlight_godot.dll`, `libportlight_godot.so`, `libportlight_godot.dylib`), so a stale copy next to an export is easy to load if you keep relying on the file-name fallback.

### 2. Minimal `export_presets.cfg`

Still absent. Do not treat this note as adding one.

Godot 4.7 names, from the 4.7.2 sources: Windows platform string `"Windows Desktop"` (`platform/windows/export/export.cpp`), macOS `"macOS"` (`platform/macos/export/export_plugin.h`). Texture keys are `texture_format/s3tc_bptc` and `texture_format/etc2_astc`. The Linux preset of this shape was exported. The Windows and macOS blocks were not exported. `application/export_angle=2` and `application/export_d3d12=2` are No, so a Windows export does not try to bundle ANGLE or the D3D12 agility SDK. `codesign/codesign=1` is built-in ad-hoc. `notarization/notarization=0` is off. `export/distribution_type=0` is Testing.

```ini
[preset.0]

name="Windows Desktop"
platform="Windows Desktop"
runnable=true
export_filter="all_resources"
script_export_mode=2

[preset.0.options]

custom_template/debug=""
custom_template/release=""
debug/export_console_wrapper=1
binary_format/embed_pck=false
texture_format/s3tc_bptc=true
texture_format/etc2_astc=false
binary_format/architecture="x86_64"
codesign/enable=false
application/export_angle=2
application/export_d3d12=2

[preset.1]

name="macOS"
platform="macOS"
runnable=true
export_filter="all_resources"
script_export_mode=2

[preset.1.options]

export/distribution_type=0
binary_format/architecture="universal"
custom_template/debug=""
custom_template/release=""
debug/export_console_wrapper=1
application/bundle_identifier="org.portlight.bounty"
codesign/codesign=1
notarization/notarization=0
```

Install official templates in `export_templates/4.7.2.stable/` (or `4.7.stable` if the editor is 4.7.0 rather than 4.7.2; Mike's editor was `4.7.stable`). `--export-debug` selects the debug library, which is the one Mike already loaded in the editor. `--export-release` selects `windows.release.x86_64` (`res://../target/release/portlight_godot.dll`) and is the untested one.

macOS arm64 and universal exports also require ETC2/ASTC VRAM import (`rendering/textures/vram_compression/import_etc2_astc`). x86_64 macOS checks S3TC/BPTC. `godot/project.godot` sets neither. S3TC defaults on. ETC2 defaults off. An arm64 or universal macOS export of the current project should fail that check until ETC2/ASTC is enabled. That check was read from Godot 4.7.2 source. It was not run on a Mac.

### 3. Toolchain fix

Known, and now demonstrated on Windows: `cargo +1.98.1` builds `portlight-godot` while `rust-toolchain.toml` stays at 1.83. The same 1.98.1 compiler (48a229cea 2026-09-01) ran the sim test suite on Linux, including `portlight-godot`'s 7 tests.

`rust-version` does not select a compiler. One Cargo invocation uses one rustc. A nested `rust-toolchain.toml` under `crates/portlight-godot` does not apply when Cargo is started at the repo root.

**Fix:** set `rust-toolchain.toml` `channel` to `1.98.1` (floor is 1.94, because that is `godot` 0.5.5's MSRV and the edition-2024 requirement). Then a plain `cargo build -p portlight-godot` is the debug library the editor loads, and the `+1.98.1` override can go away. The sim's `indexmap = "=2.11.4"` pin can stay; it built and its tests passed on 1.98.1.

Until that file changes, the working commands are:

```text
cargo +1.98.1 build -p portlight-godot
cargo +1.98.1 build --release -p portlight-godot
```

The first is what the editor needs. The second is what a release export needs. Leaving the pin at 1.83 and remembering the override works, and it is what Mike used. It is also how the release-only failure happens: `cargo build` without the override does not produce a DLL Cargo 1.83 can finish, and `cargo +1.98.1 build --release` produces a DLL the editor does not open.

### 4. Windows target

**`x86_64-pc-windows-msvc`.** That is the host Mike used, and the debug DLL from that host loaded into official Godot 4.7 (a MinGW editor). The GDExtension entry point is the C symbol `gdext_rust_init`. The `.gdextension` file looks in `target/debug/portlight_godot.dll`, which is where a default Windows `cargo build` writes. A gnu `--target` would write under `target/x86_64-pc-windows-gnu/` and miss that path.

The Linux gnu cross-link is not the recommended artifact. It used win32-thread MinGW and `msvcrt.dll`, and it was never loaded.

The msvc DLL imports `VCRUNTIME140.dll`. Mike's machine had it. A clean player machine may not. For a shipped game, install the VC++ 2015–2022 redistributable, or link the CRT statically with `RUSTFLAGS="-C target-feature=+crt-static"`. Static CRT was not built.

### 5. macOS: what a dev build skips, and what a universal export needs

No macOS binary was produced. Every macOS cell is UNTESTED. This section is the procedure, not a run.

**A dev build on one Mac skips lipo, codesign, and notarization.**

```text
cargo +1.98.1 build -p portlight-godot
```

That writes `target/debug/libportlight_godot.dylib` for the host arch. `godot/portlight.gdextension` points `macos.debug`, `macos.release`, `macos.debug.arm64`, and `macos.release.arm64` at `res://../target/{debug,release}/libportlight_godot.dylib`. On Apple Silicon the editor matches `macos.debug.arm64`. On Intel it matches `macos.debug`. Both keys name the same file, which is correct when that file is the one architecture you just built. It is the same rule as Windows: the editor wants the **debug** dylib, not the release one.

An unsigned dylib is the normal editor setup. Gatekeeper applies to apps downloaded by other people, not to a dylib you just compiled and that the local editor opens. That load was not run here.

**Universal, signing, and notarization wait until you export for someone else.**

A universal export template contains both Intel and Apple Silicon. An arm64-only dylib fails on Intel, and the other way around. `--target` also changes the output directory, so the `.gdextension` path `res://../target/release/libportlight_godot.dylib` will not see `target/aarch64-apple-darwin/release/` unless you lipo back to `target/release/`:

```text
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo +1.98.1 build --release --target aarch64-apple-darwin -p portlight-godot
cargo +1.98.1 build --release --target x86_64-apple-darwin -p portlight-godot
lipo -create -output target/release/libportlight_godot.dylib \
  target/aarch64-apple-darwin/release/libportlight_godot.dylib \
  target/x86_64-apple-darwin/release/libportlight_godot.dylib
```

`lipo` was not run. It has to be run on macOS (or with a macOS SDK); this Linux VM cannot produce those dylibs.

Codesign the exported `.app`, not the dev dylib. Godot's macOS preset can ad-hoc sign (`codesign/codesign=1`). In the 4.7.2 exporter, an ad-hoc signature that embeds a dynamic library turns on the disable-library-validation entitlement so the dylib is allowed to load. That is enough to run the export on the build machine. It was not run.

Notarization is for Gatekeeper on other people's Macs. Set `notarization/notarization` to Xcode notarytool and use a Developer ID Application certificate. The Apple Developer Program is required. The godot-rust macOS page says to sign the library by itself only when you ship the library; a full game is signed at export. Notarization can wait until a release. It was not run.

## CI

`.github/workflows/build-check-win-mac.yml` is on this branch and was not pushed. It did not run. There is no Actions URL.

`git push -u origin cursor/win-mac-build-check-5311` failed. The VM GitHub credential expired at 2026-09-28 22:21:44 UTC. GitHub returned `Invalid username or token`. There is no draft pull request from this agent. Windows evidence above is Mike's machine. macOS was not run anywhere.