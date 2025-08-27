## Hyprpaper GUI

A High-Performance GUI wallpaper selector for Hyprpaper, built with Rust using `eframe/egui`.

## About author and this Project

Hyprpaper-GUI, I created this in my free time, so it’s my exploration stuff. There are some others who created the same concept; some of them used Python Pygame, which in my experience uses high CPU and opens a little bit slowly, so I created my own version with Rust. Basically, it’s just another attempt inspired by others.

## High-Performance GUI

Instead of targeting the main folder and rendering everything, which could take a (whopping 40 seconds attempt before), this one basically:
- Tracks both ~/Pictures/Wallpapers and ~/.local/cache/thumbnails
- Lists all MD5s and renders cached images linked to ~/Pictures/Wallpapers, so it has a really responsive GUI

## Features
- High Performance GUI with Rust and egui
- Minimalistic design, Gives user full control of the app's sizing for their own Hyprland ricings via hyprland.conf
- Browse and preview image thumbnails efficiently via md5 and local cache
- Select which monitor to apply the wallpaper to
- Scans folder ~/Pictures/Wallpapers and any folders underneath and gives user album separation in the app  

## NOTICE
- Make sure you set ~/config/hypr/hyprpaper.conf "ipc = on" so the application can call "hyprctl hyprpaper ...". Otherwise, the command won’t find the Hyprpaper socket.

- For convenience, place all of your wallpapers in ~/Pictures/Wallpapers.

## Todo List
- App Memory
- Scroll Selector (Horizontal n Vertical) for your custom setups 
- Dynamically change hyprpaper.conf so the wallpaper resist even after restart + dynamic multi-monitor setups (Or maybe Varxy can do this dynamic update by updating hyprpaper)

## Screenshots
![Hyprpaper GUI Screenshot](docs/hyprpaper-gui_hyprshot_1.png)
![Hyprpaper GUI Screenshot](docs/hyprpaper-gui_hyprshot_2.png)

## Building & Running

```bash
git clone https://github.com/Kokuse557/Hyprpaper-GUI.git
cd Hyprpaper-GUI

# Debug build
cargo run

# Release build (optimized)
cargo build --release
./target/release/hyprpaper-gui
