# Slint lessons

Translated from `../essais-slint/LECONS-SLINT.md`. Paths, identifiers and on-screen labels stay in French, as in the code and the interface. Status labels: `PM` (missing punch), `Inconnue` (unknown), `Abandon` (retired), `En course` (racing), `À vérifier` (to check).

One section per trial. Goal: learn how to ask Claude Code for a faithful interface.

## Template
### Trial N — title (date)
- **Worked**:
- **Failed / gap with the mockup**:
- **Instruction that worked**:
- **Slint pitfall**:
- **For Severin** (mobile, performance, limits):

### Trial 1 — Base window (2026-10-23)
- **Approved by Robin (2026-11-02)**: screenshots OK; header button colors and dark header without a line accepted. "Auto" tested with the Mac in dark mode: OK.
- **Worked**:
  - IBM Plex Sans 1.1.0 font (IBM's GitHub repository) embedded in the program: 4 `.ttf` files in `ui/polices/` + OFL license. Plex rendering checked on a screenshot.
  - Colors checked to the pixel on the screenshots: header `#163628` / `#17241C`, background `#F5F8F4` / `#0F1A14`, card `#FFFFFF` / `#17241C`, dark outline `#24352B` all the way around.
  - Auto → Light → Dark toggle (pill button). "Auto" does follow the Mac in light mode; **not tested in system dark mode** (the Mac setting has to be changed by hand).
  - Nothing cut off: in width, the window does not go below the content's minimum width (≈ 270 px) and text wraps; in height, the content area scrolls.
- **Failed / gap with the mockup**:
  - First render: the header took 40% of the height (see pitfall 2).
  - Header button colors (translucent white in light mode, `#1F2F26` / `#24352B` / `#2C4034` in dark mode): **not set by the design guide**, chosen for the trial. To be approved.
  - Light "secondary text" = "grey-green" with no exact value: not used for now.
  - No line under the header in dark mode (it would be a one-sided border): only the difference in tint from the background separates it.
- **Instruction that worked**: give the exact light/dark values in a table + "the dark header is neutral `#17241C`"; centralize everything in `global Theme` with a computed `sombre` property.
- **Slint pitfall**:
  1. Fonts: `import "polices/IBMPlexSans-Regular.ttf";` in a `.slint` file is enough to embed them; then `default-font-family: "IBM Plex Sans"` on the `Window`.
  2. In a `VerticalLayout`, a `Rectangle` (header) stretches by default → set `vertical-stretch: 0` on the header and `vertical-stretch: 1` on the content.
  3. Wrapped text (`wrap: word-wrap`): the window's minimum height is computed for the normal width, not the reduced width → text cut off at the bottom if the window is narrow **and** short. Solution: content in a `Flickable` (`content-height: max(self.height, contenu.preferred-height)`).
  4. Following the system: `Palette.color-scheme` (from `std-widgets.slint`). To force a mode without losing "auto", keep your own `mode` in `Theme` rather than writing to `Palette`.
  5. Several trials in a single project: `build.rs` with `slint_build::compile_with_output_path` (one generated file per trial) + `include!` in each `src/bin/essaiN.rs`.
  6. Screenshots: `window().take_snapshot()` gives the exact image (×2 on a Retina display), with no macOS permission. Option `--capture clair|sombre|auto chemin [largeur hauteur]`.
  8. Compile warning: `viewport-width` / `viewport-height` on `Flickable` are deprecated → use `content-width` / `content-height`. Claude Code sometimes knows the old syntax: ask it to deal with every warning.
  7. `cargo` is not in the PATH of every terminal: `~/.cargo/bin/cargo` (or `source ~/.cargo/env`).
- **For Severin**: Slint 1.18.1, default renderer and backend. First compile ≈ 1 min 40 s (559 dependencies), then a few seconds. Fonts: ≈ 800 KB for 4 weights (can be reduced to the Latin alphabet later).

### Trial 2 — Theme and swatch page (2026-11-03)
- **Robin's decisions**: "Inconnue" moves from Problème to À vérifier (`references/charte.md` changed, as on `08-direct.png`); swatch page with the word alone, icons come in trial 3; missing design guide values accepted as proposed.
- **Worked**:
  - Complete `ui/theme.slint`: colors, 5 families, 7 text sizes, weights, touch targets, radii. No hard-coded values elsewhere (automated search: only `/ 1px` remains, which converts a size into a number to display).
  - The 5 families compared by script with the design guide's lines: compliant. Colors read to the pixel on the swatch page screenshots: compliant.
  - The swatch page shows values read from `Theme` (through Rust), not copied values: it cannot lie about the theme.
  - Trial 1: no more warnings, screenshots byte-identical after the fix.
  - Plex digits: all 600 wide (read from the fonts); nothing to adjust to align times.
- **Failed / gap with the mockup**:
  - A single screenshot cannot show the whole page: macOS limits the window to the screen height (978 px here). Hence 3 screenshots per mode.
  - Minimum width of the swatch page ≈ 800 px: nothing is cut off, but the page does not rearrange. To be handled for the phone (trial 4).
  - Badge (height 22) and block (80 × 54, radius 12) dimensions estimated from `08-direct.png`, not given by the design guide: to be confirmed in trial 3.
- **Open values**:
  - Read from the screenshots: light secondary text `#4A5D52`, light thin outline `#C9D6CE`.
  - Proposed then accepted: dark thin outline `#24352B`, dark tab text `#0F1A14`, dark background for badges and blocks `#1F2F26`, burgundy band `#FFFFFF`.
  - Light secondary button: the design guide says both inner surface and "white"; white kept, as on the screenshots.
  - **Still to be defined**: "À vérifier" block in light mode; field being edited in dark mode.
- **Instruction that worked**: reread the screenshots before proposing a value (three places gave exactly the same `#4A5D52`); ask for the origin of each value to be stated (design guide, read from a screenshot, proposed).
- **Slint pitfall**:
  1. `cargo build -q` hides Slint's warnings: they go through `build.rs`. Always compile without `-q` and read the output.
  2. Slint cannot write a color as `#RRGGBB`: `pure callback hexa(color) -> string` in a `global`, provided by Rust with `on_hexa`.
  3. A `Flickable` does not pass its content's minimum width on to the window: add `min-width: contenu.min-width`.
  4. `GridLayout` does not accept `for`: a repeated row of columns = `VerticalLayout` + `HorizontalLayout` with fixed widths.
  5. To test scrolling without a hand on the mouse: `window().dispatch_event(WindowEvent::PointerScrolled { .. })` then a screenshot.
- **For Severin**: swatch page in two fixed columns, not suited to the phone as it stands (see above).

### Trial 3 — Components (2026-11-03)
- **Worked**:
  - `ui/composants/`: `boutons`, `statut` (badge and block), `onglets` (with Direct), `carte`, `champ`, `ligne-liste`, `icones`, `bouton-mode`. Trial 2 now reuses the card, the badge and the block.
  - 26 icons in `ui/icones/`: 19 Lucide 1.67.0 (license copied) + 7 orienteering icons drawn from the design guide.
  - New values from Robin integrated: light À vérifier block, dark fields. Swatch page updated, no more "to be defined".
  - Measurements close to the screenshots: tab bar 448 px (445 on the screenshot), Direct 87 (86), "Arrivée manuelle..." about 162 (162).
  - Tested with simulated events: tab change, Direct, Theme button, focus in a field, keyboard typing. Trial 1 still byte-identical.
- **Approved by Robin by hand (2026-11-03)**: hover, tab clicks, input field, size of "Attribuer".
- **Robin's decisions after review (2026-11-03)**:
  - Field: design guide style (solid background with no outline at rest), 36 px desktop, 48 px phone. Deliberate gap with `02-participants.png` (white with a thin outline).
  - Primary button: two heights (36 and 48), 18 bold text for contrast. Deliberate gap with the screenshots (about 14 px).
  - Action in a row ("Attribuer"): 13 on 24 px judged too small, 18 on 36 px judged too big, kept: 16 bold, 32 px drawn (desktop) or 40 px (phone), click target unchanged (36 or 48).
  - Row: both forms. Table on desktop (`08-direct.png`), block › name › time row on phone (design guide).
  - Inconnue and Abandon icons kept, En course dot kept (`footprints` not used for this status).
  - Line between rows: accepted. Subtle button: underlined text.
  - PM time: Robin's idea, greyed, lighter weight and struck through, in both forms. Deliberate gap with `08-direct.png`.
- **Failed / remaining gap**:
  - Table fitted to the 1100 px window: column widths reduced, no sort arrows (▾, outside the design guide and with no logic here).
  - `references/charte.md` updated on 2026-11-03 with these decisions (at Robin's request).
- **Choices with no value in the design guide**: hover and press tints; tabs, separator and Direct halo in dark mode; error field in light mode (white background); small "···" button written as text, taken from `08-direct.png`.
- **Instruction that worked**: enlarge the screenshots and read the edges to the pixel before coding; give Robin the design guide / screenshot contradictions rather than silently choosing one.
- **Slint pitfall**:
  1. Lucide renamed `circle-help` to `circle-question-mark`: file kept under the design guide's name.
  2. A Lucide SVG (`currentColor`) comes out black; color it with `colorize` on `Image`.
  3. In a `HorizontalLayout`, columns of different heights stretch and their contents shift: set `alignment: start` in the component.
  4. `@image-url` in a `global` lets you group the icons and pass them in a `struct`.
  5. `for x in [...]` cannot be used in `GridLayout`: rows of `HorizontalLayout`.
  6. Testing without the mouse: `dispatch_event` with `PointerPressed` / `PointerReleased` / `KeyPressed`.
  7. Underlined and struck-through text: no option found on `Text`; a `Rectangle` line drawn under or over the text, at its width.
- **For Severin**: minimum page width ≈ 936 px (icon rows): the page will have to rearrange for the phone (trial 4). Field and primary button already have a `telephone` property (48 px); tabs, Direct and secondary buttons stay at 36 and will need adapting.

### Trial 4 — On the phone, iPhone part (2026-11-03)
- **Done on the simulator (iPhone 17 Pro, iOS 26.5)**: `essai4` page in one column, embedded font and icons, safe area, touch scrolling, virtual keyboard (the field stays above it, closed with Return or by touching the page), tabs, Direct, auto theme following the iPhone, light and dark.
- **Not done yet**: real iPhone (signing with Robin's Apple account); Android postponed by Robin.
- **Installed**: Rust targets `aarch64-apple-ios` and `aarch64-apple-ios-sim`, XcodeGen 2.46.0 (Homebrew): under a minute. Xcode 26.6 and the simulator were already there. Skia is fetched by Cargo, with no separate install.
- **Setting changed**: `ConnectHardwareKeyboard = false` for the simulator, otherwise the virtual keyboard does not appear.
- **Time**: about 20 minutes end to end; first iOS compile 53 s, then 10 to 15 s.
- **Gaps with Mac and the mockup**:
  - No table on the phone (Robin's decision); blocks and badges on two rows.
  - Icon tabs, with the word only on the active tab (`09-telephone-boitiers.png`). Inactive tabs: at least 44 px wide, 48 high.
  - Direct: live mint dot; the red dot on the screenshot is not reused.
  - Minimum width tested: 375 px. Below that, the tab bar no longer fits.
- **Instruction that worked**: test at 375 px wide ("Participants" overflowed) and check that a compile is not abnormally fast.
- **Slint and iOS pitfall**:
  1. `project.yml` template from the documentation: `outputFileLists` fails; `outputFiles` is needed.
  2. With `outputFiles`, Xcode skips cargo if nothing has changed from its point of view (1 s compile, old version tested): add `basedOnDependencyAnalysis: false`.
  3. Slint does not move a field above the keyboard: `virtual-keyboard-position`, `virtual-keyboard-size` and the `Flickable`'s `content-y`.
  4. Touching elsewhere does not close the keyboard: `clear-focus()` on Return, and a `TouchArea` behind the page.
  5. Pasting text with the simulator's tool makes iOS believe a physical keyboard is connected: the virtual keyboard disappears until the simulator restarts.
  6. A `width` computed from `parent.width` in a row creates a calculation loop: fixed width taken from `Theme`.
  7. The app takes a few seconds to appear in the debug build (black screen). Not measured in the release build.
- **For Severin**: iPhone simple to set up, Rust only on iOS (no C++). Release build size: 25 MB, 9.6 MB compressed (excluding the App Store's file size reduction, not measured). To check: the terms of the chosen Slint license for a published app.

### Trial 4 (continued) — Rows to the edge and animations (2026-11-03)
- **Robin's decisions**: row background and lines to the edge of the card, content at 8 px (phone and table); sliding tab pill; other animations added right away, without overdoing it.
- **Robin's correction**: the 8 px margin broke the alignment. Only the background and lines go to the edge; the content takes the title's margin again (24 px desktop, 16 px phone).
- **Approved by Robin on screen**: the animations.
- **Done**:
  - Tabs: pill slides in 220 ms, on Mac and iPhone; on iPhone the word opens and the neighboring tabs shift together.
  - Breathing dot: En course, Direct.
  - Smooth tints (120 ms): buttons, hover, fields.
  - Smooth page lift when the keyboard opens.
  - All turned off by Reduce Motion.
- **Checked**:
  - Screenshot 100 ms after a click: the pill is indeed moving (Mac and phone page).
  - Reduced setting, simulator: 0 pixels different between two frames, 84 otherwise, on the Direct button.
- **Not checked**: smoothness by eye and on a real iPhone (for Robin to judge); the reduced setting on the Mac side (system setting not changed).
- **Not done**:
  - Spinning icon: no "reading in progress" element on these pages, and no loading icon in the design guide.
  - Light / dark transition: instant change.
- **Instruction that worked**: describe the expected effect (the pill slides) and capture during the movement, not only at the end.
- **Slint pitfall**:
  1. The position of a `for` element cannot be read from outside: each tab reports its place (`init`, `changed x`, `changed width`). On the phone, the bar computes everything itself, otherwise the pill chases values that move on every frame.
  2. `animate` accepts a computed duration: 0 ms at launch (no stray animation) and with the reduced setting.
  3. `clip: true` on a rounded `Rectangle` clips its children to the rounding.
  4. No Reduce Motion setting found on the Slint side: read from Rust (`UIAccessibilityIsReduceMotionEnabled` on iOS, `defaults` on Mac), reread every 3 s.
  5. `animation-tick()` redraws continuously while the dot is displayed: to watch for phone battery.

### Trial 4 (end) — Controlling the keyboard (2026-11-03)
- **Robin's request**: a field that expects a number must open the numeric keyboard.
- **Finding**: on iOS, `input-type: InputType.number` does not change the keyboard. Slint goes through winit's `set_ime_purpose`, and winit ignores it (message `set_ime_purpose is ignored on iOS`). On Android: not tested.
- **Solution**: `essais_slint::choisir_clavier` (`src/lib.rs`) adds to the iOS view the `keyboardType` method that UIKit reads, then calls `reloadInputViews`. `Champ` notifies Rust on each focus through the `Clavier` global.
- **Checked on the simulator**: number pad on the chip field, digit typed, back to the letter keyboard on another field, without closing the keyboard.
- **Not checked**: on a real iPhone (signing done, app launch to be confirmed by Robin).
- **Limits**: about thirty `unsafe` lines that replace a method in a winit class: to be reviewed at each Slint update. The iPhone number pad has no Return key: close it by touching the page. Input remains limited on the Slint side by `input-type`, not tested with letters.
- **For Severin**: to report to Slint (standard setting missing on iOS). Point to look at before choosing Slint on mobile: whatever is not provided for goes through system-specific code.

### Trial 5 — The Direct screen (2026-11-03)
- **Robin's choice**: Direct, because it also serves trial 7.
- **Worked**:
  - Desktop version overlaid on `08-direct.png` (`captures-essais/essai5-superposition-08-direct.png`): columns, table rows, banner, course diagram and indicators land in the same places.
  - A single page that chooses its version by width (threshold 1100 px). Nothing cut off at 1100 and 375 px.
  - All data comes from Rust (`DonneesDirect`); Slint only displays. Ready for trial 7.
  - On the simulator: safe area, scrolling, keyboard on search, closing the keyboard, light and dark.
- **Failed / gap with the mockup**:
  - Deliberate (Robin's decisions): "Attribuer" and "Régler" in 16 bold; search field in the design guide style; icons on Inconnue; PM time struck through.
  - Small text at 13 px (design guide minimum) instead of about 11 px: header, readers, diagram, hence a slight width shift. Diagram 76 px high instead of 72.
  - Absent because outside the design guide or off-screen: search magnifier (`search` not in the icon list), sort and collapse arrows (▾, absent from Plex), the mockup's "Démo" button. Icon added to "Déjà lue" (`triangle-alert`), absent from the mockup.
  - Banner stats aligned to the end of the bar, since the arrow is missing.
  - Phone: no reference screenshot, composed from `09-telephone-boitiers.png`. Title and indicators on two rows so as not to cut off "Régionale de Lyon"; reviewed by eye only. No Theme button: follows the system.
  - Phone list title ("8 dernières lectures"): added, to be approved.
- **Instruction that worked**: overlay the render and the screenshot at 50%: the gaps jump out and get fixed in one pass.
- **Slint pitfall**:
  1. An empty `Rectangle` at `horizontal-stretch: 1` is not enough to push its neighbors: elements that inherit from `Rectangle` stretch too, they need `horizontal-stretch: 0`.
  6. iOS keeps the keyboard's height when it is hidden (placed below the screen): also test `virtual-keyboard-position.y`, otherwise a large gap remains under the page. The defect already existed in trial 4.
  2. Table: positions as a share of the width (`x`), with no layout, so that the header and rows stay aligned.
  3. Going from a Rust `enum` to a color or a status: `pure public function` in a Slint `global`; the values stay in `Theme`.
  4. `rotation-angle` is deprecated: `transform-rotation`.
  5. `Text` does not clip itself if its parent has no layout: it overflows silently. Check at small widths.
- **For Severin**: one page holds both desktop and phone, but these are two compositions to write and maintain. iOS launched from `ios/` now compiles `essai5`.


### Robin's feedback on a real iPhone (2026-11-03)
- Direct screen (`essai5`) launched from Xcode on their iPhone: "the display is spot on", perfect once launched.
- **Stutter at startup, resolved**: in Release, the app kept stuttering with "Debug executable" checked, and was fast from the start once unchecked (Robin's observation by eye, not timed). The Xcode debugger was the cause. To judge smoothness: Release, "Debug executable" unchecked.
- **Not yet judged**: interactions, since almost nothing is active on this screen.
- **Pitfall**: recreating the project with `xcodegen` erases the Team chosen in Xcode; put it in `project.yml`.
- **For Severin**: rendering on a real device is confirmed for this screen.

### Trial 6 — Animations (2026-11-03)
- **Robin's instruction**: icons do not matter, this is a Slint test; several can be used. Test icons outside the design guide, in `ui/icones/test/`, design guide unchanged.
- **Done**: `IconeTournante` component, `essai6` page (8 spinning icons, breathing dot, tabs, setting state), "Lecture en cours" reader on Direct.
- **Checked**:
  - iPhone simulator, pixels changed between two frames: about 20,000 without the setting, 0 with Reduce Motion, about 20,000 once the setting is removed. Icons frozen, no restart needed.
  - CPU on Mac, release build: about 5% with the animations, about 0% on a still page.
- **Approved by Robin on Mac**: smoothness "top" (page still too sparse for a real load test); Reduce Motion works.
- **Not checked**: CPU and battery on a real iPhone; smoothness under load.
- **Instruction that worked**: compare two frames a moment apart: more reliable than a still screenshot to prove that an animation runs or stops.
- **Slint pitfall**:
  1. `transform-rotation` rotates an `Image` around its center; the angle comes from `animation-tick()`.
  2. `animation-tick()` redraws constantly while the element is displayed: display it only when useful (reader actually reading).
  3. To change the iOS binary without losing the account: edit `project.yml` **and** `project.pbxproj`, without rerunning `xcodegen`.
- **For Severin**: animations are simple to write. Cost to be measured on the phone.

### Trial 7 — Rust provides, Slint displays (2026-11-03)
- **Requested by Robin on top of `ESSAIS.md`**: 500-row test, animated arrival, Rafale button (test only).
- **Done**: a Rust thread simulates the readers; the main thread applies the rules (known runner, already read, PM, place, counters); Slint displays and sends clicks back. Most recent on top, 5000 rows maximum.
- **Checked**:
  - Simulator: "Attribuer" transforms the row (Puce 9147066 becomes BERNARD Anna, OK, 106e); Rafale; auto theme light and dark; stable view when scrolled (504 then 506 rows, same positions).
  - Mac, release build: CPU about 8% at rest, 15% in a burst, the same at 500 and 5000 rows; memory 145 MB; preparing 5000 rows: 3 ms, 20000: 8 ms (data only).
  - Trial 5 after grouping the code: phone version pixel-identical; desktop version: 965 pixels different out of 5 million, attributed to animations in progress (not isolated one by one).
  - iPhone release app: 26 MB.
- **Approved by Robin on screen, on Mac and a real iPhone**: "it works very well everywhere".
- **Not measured**: CPU and battery on the phone.
- **Deliberate gaps**:
  - Phone: condensed banner (no readers or diagram) and no search or "Arrivée manuelle", to leave room for the list, which scrolls on its own.
  - "Attribuer" picks a dummy runner, with no selection window. Places given by order of arrival, not by time.
  - Still inactive: search, "Arrivée manuelle", "···", tab pages.
- **Instruction that worked**: reread a test's log: the "5000 rows" measurement was actually running with 500; and recheck a click afterwards: the first "Attribuer" test targeted a row that had moved.
- **Slint pitfall**:
  1. `ListView` only creates the visible rows: essential for long lists, provided its rows have a fixed height. It does not nest well in a page that already scrolls.
  2. `ListView` keeps the view in place when inserting at the top: do not compensate `content-y`, otherwise the view drifts by one row with each arrival.
  3. Slint models do not leave the main thread: another thread goes through `upgrade_in_event_loop`.
  4. `changed` reacts to a property of the component: first copy the `global`'s value into a local property.
  5. The slide is computed with `animation-tick()` only during the arrival, then cut off by a `Timer`.
- **For Severin**: the Rust/Slint architecture holds on Mac, including in bursts. CPU stays at 8% at rest because of the continuous animations: to be measured on the phone.
