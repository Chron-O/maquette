# Chron'O — interface mockup

Interface mockup of Chron'O (orienteering timing software, SportIdent), in Rust and [Slint](https://slint.dev).
**Discussion support, not a specification.** All names, clubs and card numbers are invented.

- **Web version**: https://chron-o.github.io/maquette/
- **Frozen HTML prototype** (in `html/`): https://chron-o.github.io/maquette/html/

Main project: [Chron-O/chrono](https://github.com/Chron-O/chrono).

## Run it

Install [Rust](https://rustup.rs), then:

```bash
cargo run
```

The layout follows the window width (narrow under 600 px, medium up to 1000 px, wide above) and the theme follows the system. The **Démo** button forces touch or mouse sizes.

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

Saves a screen's 8 captures (4 formats, light and dark) in `captures/`, on macOS:

```bash
scripts/captures.sh 00-evenement onglet 1
```

## Layout of the code

- `src/`: state and data (Rust). Made-up data lives in `src/donnees/`.
- `ui/`: what is displayed (Slint). Every design value comes from `ui/theme.slint`.
- `references/`: visual charter and captures of the HTML prototype.

Some names are in French, like the charter they come from.

## License

GPLv3. IBM Plex Sans: OFL (`ui/polices/`). Lucide icons: ISC (`ui/icones/`).
