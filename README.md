# Chron'O — interface mockup

Interface mockup of Chron'O (orienteering timing software, SportIdent), in Rust and [Slint](https://slint.dev).
**Discussion support, not a specification.**

The app shows no made-up data by default: it is empty until you turn on the test data generator (**Démo** button, then *Données de test*). The generator stands in for connected devices and real events; all its names, clubs and card numbers are invented.

- **Web version**: https://chron-o.github.io/maquette/
- **Frozen HTML prototype** (in `html/`): https://chron-o.github.io/maquette/html/

Main project: [Chron-O/chrono](https://github.com/Chron-O/chrono).

## Run it

Install [Rust](https://rustup.rs), then:

```bash
cargo run
```

The layout follows the window width (narrow under 600 px, medium up to 1000 px, wide above) and the theme follows the system. On-screen text is in French. The **Démo** button also forces touch or mouse sizes.

## Other targets

**Web** (needs the `wasm32-unknown-unknown` target and `wasm-pack`). This builds `site/`, which GitHub Pages publishes on every merge to `main`:

```bash
scripts/site.sh
```

```bash
python3 -m http.server 8123 --directory site
```

**iPhone and iPad** (needs Xcode, XcodeGen and the `aarch64-apple-ios` and `aarch64-apple-ios-sim` targets):

```bash
cd ios && xcodegen && open Maquette.xcodeproj
```

To run on a real device, first create `ios/Local.xcconfig` with `DEVELOPMENT_TEAM = YOUR_TEAM_ID` (ignored by git).

## Captures

Saves a screen's 8 captures (4 formats, light and dark) in `captures/`, on macOS. With `CAPTURE_TEST_DATA`, the test data comes from a fixed seed:

```bash
CAPTURE_TEST_DATA=1 scripts/captures.sh 01-home-test-data
```

## Layout of the code

- `src/`: state (Rust). The test data generator is `src/test_data.rs`.
- `ui/`: what is displayed (Slint). Every design value comes from `ui/theme.slint`. The logo is in `ui/logo/`, the map illustration in `ui/map/`.
- `references/`: visual charter and captures of the HTML prototype.

## License

GPLv3. IBM Plex Sans: OFL (`ui/fonts/`). Logo lettering outlined from Instrument Sans: OFL. Lucide icons: ISC (`ui/icons/`).
