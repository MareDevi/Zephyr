# Tauri + React + Typescript

This template should help get you started developing with Tauri, React and Typescript in Vite.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Troubleshooting logs (for bug reports)

1. Start the app with backend logs enabled:
   - `RUST_LOG=Zephyr_lib=debug,zbus=info bun run tauri dev`
2. Reproduce the issue in UI.
3. Copy terminal output and include:
   - failing action/page
   - full `D-Bus:` error lines
4. Collect service status on Linux:
   - `systemctl status asusd supergfxd power-profiles-daemon --no-pager`
   - `journalctl -u asusd -u supergfxd -u power-profiles-daemon -n 200 --no-pager`
