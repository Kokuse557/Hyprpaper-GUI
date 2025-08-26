# Hyprpaper GUI

A High-Performance GUI wallpaper selector for Hyprpaper, built with Rust using `eframe/egui`.


About Hyprpaper-GUI: I created this in my free time, so it’s my exploration stuff. There are some others who created the same concept; some of them used Python Pygame, which in my experience uses high CPU and opens a little bit slowly, so I created my own version with Rust. Basically, it’s just another attempt inspired by others.

## High-Performance GUI

Instead of targeting the main folder and rendering everything, which could take a (whopping 40 seconds attempt before), this one basically:
- Tracks both ~/Pictures/Wallpapers and ~/.local/cache/thumbnails
- Lists all MD5s and renders cached images linked to ~/Pictures/Wallpapers, so it has a really responsive GUI

## Features
- High Performance GUI with Rust and egui
- Browse and preview image thumbnails
- Cleans up with efficient caching and preloading logic
- Select which monitor to apply the wallpaper to
- Notifications and error overlays

## NOTICE
- Make sure you set ~/config/hypr/hyprpaper.conf "ipc = on" so the application can call "hyprctl hyprpaper ...". Otherwise, the command won’t find the Hyprpaper socket.

- For convenience, place all of your wallpapers in ~/Pictures/Wallpapers.

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
