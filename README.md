# Discord Idle Picker

A Windows desktop app that makes Discord show selected games as "Playing" without running the games. It starts lightweight dummy processes that satisfy Discord's game detection. The UI and workflow follow [Steam Idle Picker](https://github.com/4sou9/Steam_Idle_Picker).

![Screenshot](01.png)

## Requirements

- Windows 10/11 (x64)
- Discord desktop app
- Steam (only for games Discord detects through Steam)

## Usage

1. Launch `Discord Idle Picker.exe`. On first launch the list of detectable games is downloaded (about 19,000 games; refreshed every 24 hours or with the refresh button)
2. Check the games you want to show (up to 32) — checked games stay pinned to the top, followed by favorites
3. Click the play button. Discord shows the games as "Playing"
4. Click the same button (now a stop icon) to stop everything, or uncheck a game to stop just that one

- **EXE / STEAM badge**: how the game is detected. EXE games run from `%LOCALAPPDATA%\com.discordidlepicker.app\runtime`. STEAM games run from their Steam folder; for games that are not installed, a temporary folder and Steam registration are created and the registration is removed as soon as Discord detects the game
- **✓ / !**: ✓ means Discord detected the game. ! means Discord did not react within 15 seconds. Games that are not Discord's "main" game can take up to 5 minutes to get ✓
- **☆**: hover a row and click the star to add a favorite. Use the filter next to the search box to show favorites, running games, EXE or Steam games
- **Right-click a row**: favorite, copy Discord ID, open the Steam store page, open the dummy's location
- Search matches game names and aliases; digits only also match Steam AppID and Discord ID

## Notes

- If the app is killed, the dummies are terminated with it, and whatever was placed is removed the next time the app starts. If a Steam registration had to be cleaned up while Steam was running, the footer asks you to restart Steam
- Some antivirus software may flag the dummy executables because they are named after games
- Specification and verification notes: [docs/spec.md](docs/spec.md), [docs/verification-results.md](docs/verification-results.md)

## Development

Built with [Tauri 2](https://tauri.app) (Rust) + React/TypeScript.

```bash
npm install
npm run tauri dev    # run in dev mode (builds the dummy exe first)
npm run tauri build  # produce a release build + NSIS installer
```

The app icon is drawn by `scripts/make-icon.py` (Pillow). To change it, edit the script, then run `python scripts/make-icon.py` and `npx tauri icon app-icon.png`, and delete the generated files that `tauri.conf.json` does not reference (`android/`, `ios/`, `icon.icns`, `Square*Logo.png`, ...).
