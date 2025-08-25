# Hyprpaper GUI

A High-Performance GUI wallpaper selector for Hyprpaper, built with Rust using `eframe/egui`.
i created this in my freetime, so its my exploration stuff...

About Hyprpaper-GUI, there is some others that create the same concept, some of them uses python pygame, which in my experience it uses high cpu and opens lil bit slow, so i create my own version with Rust, basically just another attempt that inspired by others.

## High-Performance GUI

Instead targeting the main folder and renders everything which is able to open in whopping 40 seconds (my last attempt), this one basically :
- Tracks both ~/Pictures/Wallpapers and ~/.local/cache/thumbnails
- List all md5, and renders cached images that linked to the ~/Pictures/Wallpapers so it has really responsive GUI

## Features
- High Performance GUI with Rust and egui
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
