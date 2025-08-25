# Hyprpaper GUI

A GUI wallpaper selector for Hyprpaper, built in Rust using `eframe/egui`.

## Features
- Browse and preview image thumbnails
- Cleans up with efficient caching and preloading logic
- Select which monitor to apply the wallpaper to
- Clear wallpapers per monitor
- Notifications and error overlays

## NOTICE
- Just to make you have to set the ~/config/hypr/hyprpaper.conf "ipc = on" so the application able to call "hyprctl hyprpaper ..." 
- For convenience, place all of your wallpapers at ~/Pictures/Wallpapers

## Screenshots
*(Add images here for visual context)*

## Building & Running

```bash
git clone https://github.com/Kokuse557/Hyprpaper-GUI.git
cd Hyprpaper-GUI

# Debug build
cargo run

# Release build (optimized)
cargo build --release
./target/release/hyprpaper-gui
