# Hyprpaper GUI

A High-Performance GUI wallpaper selector for Hyprpaper, built with Rust using `eframe/egui`.

## Features
- High Performance GUI
- Browse and preview image thumbnails
- Cleans up with efficient caching and preloading logic
- Select which monitor to apply the wallpaper to
- Clear wallpapers per monitor
- Notifications and error overlays

## NOTICE
- Make you have to set the ~/config/hypr/hyprpaper.conf "ipc = on" so the application able to call "hyprctl hyprpaper ...", if not the command wont found the hyprpaper socket.
- For convenience, place all of your wallpapers at ~/Pictures/Wallpapers.

## Screenshots
![Hyprpaper GUI Screenshot](docs/hyprpaper-gui_hyprshot.png)

## Building & Running

```bash
git clone https://github.com/Kokuse557/Hyprpaper-GUI.git
cd Hyprpaper-GUI

# Debug build
cargo run

# Release build (optimized)
cargo build --release
./target/release/hyprpaper-gui
