use eframe::egui;
use egui::Vec2;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};
use image;
use rfd::FileDialog;
use md5;

struct MyApp {
    img_dir: PathBuf,
    textures: HashMap<String, egui::TextureHandle>,
    thumbnail_scale: f32,
    selected: Option<String>,
    watcher_rx: Receiver<Result<notify::Event, notify::Error>>,
    watcher: RecommendedWatcher,
    first_load: bool,
    monitors: Vec<String>,
    selected_monitor: Option<String>,
    monitor_warning_until: Option<Instant>,
    last_error: Option<(String, Instant)>,
}

impl MyApp {
    fn new(img_dir: PathBuf) -> Self {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default()).unwrap();
        watcher.watch(&img_dir, RecursiveMode::NonRecursive).unwrap();

        let monitors = Self::get_monitors();
        let selected_monitor = monitors.get(0).cloned(); // auto-select first

        Self {
            img_dir,
            textures: HashMap::new(),
            thumbnail_scale: 1.5,
            selected: None,
            watcher_rx: rx,
            watcher,
            first_load: true,
            monitors,
            selected_monitor,
            monitor_warning_until: None,
            last_error: None,
        }
    }

    fn get_monitors() -> Vec<String> {
        let output = Command::new("hyprctl").args(&["monitors", "-j"]).output();
        if let Ok(out) = output {
            if let Ok(text) = String::from_utf8(out.stdout) {
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
                if let Some(arr) = parsed.as_array() {
                    return arr
                        .iter()
                        .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                        .collect();
                }
            }
        }
        vec![]
    }

    fn reload_textures(&mut self, ctx: &egui::Context) {
        self.textures.clear();

        let thumb_dir = dirs::home_dir().unwrap().join(".cache/thumbnails/large");
        let _ = fs::create_dir_all(&thumb_dir);

        for entry in fs::read_dir(&self.img_dir).unwrap_or_else(|_| fs::read_dir(".").unwrap()) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let uri = format!("file://{}", path.to_string_lossy());
            let digest = format!("{:x}", md5::compute(uri.as_bytes()));
            let thumb_path = thumb_dir.join(format!("{}.png", digest));

            let thumb_img = if thumb_path.exists() {
                image::open(&thumb_path).ok()
            } else {
                image::open(&path).ok().map(|img| {
                    let thumb = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
                    let _ = thumb.save(&thumb_path);
                    thumb
                })
            };

            if let Some(img) = thumb_img {
                let rgba = img.to_rgba8();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [img.width() as usize, img.height() as usize],
                    rgba.as_flat_samples().as_slice(),
                );
                let texture = ctx.load_texture(path.to_string_lossy(), color_image, Default::default());
                self.textures.insert(path.to_string_lossy().to_string(), texture);
            }
        }
    }

    fn set_wallpaper(&mut self, path: &str) {
        if let Some(monitor) = &self.selected_monitor {
            let abs = std::fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));

            // Step 1: preload the wallpaper
            let preload = Command::new("hyprctl")
                .args(&["hyprpaper", "preload", &abs.to_string_lossy()])
                .output();

            if preload.as_ref().map(|o| o.status.success()).unwrap_or(false) {
                // Step 2: set the wallpaper
                let wallpaper = Command::new("hyprctl")
                    .args(&["hyprpaper", "wallpaper", &format!("{},{}", monitor, abs.display())])
                    .output();

                if wallpaper.as_ref().map(|o| o.status.success()).unwrap_or(false) {
                    // Step 3: reload hyprpaper
                    let _ = Command::new("hyprctl")
                        .args(&["hyprpaper", "reload"])
                        .output();
                } else {
                    eprintln!(
                        "⚠ hyprpaper wallpaper failed: {}",
                        String::from_utf8_lossy(&wallpaper.unwrap().stderr)
                    );
                    self.last_error = Some((
                        "Failed to apply wallpaper".into(),
                        std::time::Instant::now(),
                    ));
                }
            } else {
                eprintln!(
                    "⚠ hyprpaper preload failed: {}",
                    String::from_utf8_lossy(&preload.unwrap().stderr)
                );
                self.last_error = Some((
                    "Failed to preload wallpaper".into(),
                    std::time::Instant::now(),
                ));
            }
        } else {
            eprintln!("⚠ No monitor selected");
            self.last_error = Some((
                "No monitor selected".into(),
                std::time::Instant::now(),
            ));
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.first_load {
            self.reload_textures(ctx);
            self.first_load = false;
        }

        while let Ok(Ok(event)) = self.watcher_rx.try_recv() {
            match event.kind {
                notify::EventKind::Create(_)
                | notify::EventKind::Modify(_)
                | notify::EventKind::Remove(_) => {
                    self.reload_textures(ctx);
                    let _ = std::process::Command::new("hyprctl")
                        .args(&["hyprpaper", "reload"])
                        .output();
                }
                _ => {}
            }
        }

        // Top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Folder: {}", self.img_dir.display()));

                if ui.button("...").clicked() {
                    if let Some(folder) = FileDialog::new().set_directory(&self.img_dir).pick_folder() {
                        self.img_dir = folder;
                        self.watcher.unwatch(&self.img_dir).ok();
                        self.watcher.watch(&self.img_dir, RecursiveMode::NonRecursive).ok();
                        self.reload_textures(ctx);
                    }
                }

                if ui.button("➕").clicked() {
                    self.thumbnail_scale *= 1.2;
                }
                if ui.button("➖").clicked() {
                    self.thumbnail_scale /= 1.2;
                }

                ui.add(egui::Slider::new(&mut self.thumbnail_scale, 0.5..=3.0).text("Zoom"));

                // Monitor selection with warning flicker
                let flashing = self.monitor_warning_until
                    .map(|t| Instant::now() < t)
                    .unwrap_or(false);

                let _combo_visuals = if flashing {
                    let t = (Instant::now().elapsed().as_millis() as f32 / 200.0).sin();
                    egui::Color32::from_rgba_unmultiplied(255, 0, 0, (t * 127.0 + 128.0) as u8)
                } else {
                    ui.visuals().widgets.inactive.bg_fill
                };

                egui::ComboBox::from_label("Monitor")
                    .selected_text(
                        self.selected_monitor
                            .clone()
                            .unwrap_or_else(|| "Select".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        for m in &self.monitors {
                            if ui
                                .selectable_label(
                                    self.selected_monitor.as_deref() == Some(m.as_str()),
                                    m,
                                )
                                .clicked()
                            {
                                self.selected_monitor = Some(m.clone());
                            }
                        }
                    });
            });
        });

        // Central grid
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let target_height = 128.0 * self.thumbnail_scale;
                let padding = 4.0;
                let available_width = ui.available_width();

                let textures: Vec<(String, egui::TextureHandle)> =
                    self.textures.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

                let mut current_row: Vec<(String, egui::TextureHandle, Vec2)> = Vec::new();
                let mut row_width = 0.0;

                for (path, texture) in textures {
                    let tex_size = texture.size_vec2();
                    let scale = target_height / tex_size.y;
                    let img_size = tex_size * scale;

                    if !current_row.is_empty() && row_width + padding + img_size.x > available_width {
                        let left_padding = (available_width - row_width) / 2.0;
                        ui.horizontal(|ui| {
                            ui.add_space(left_padding);
                            for (p, tex, size) in &current_row {
                                let (rect, response) =
                                    ui.allocate_exact_size(*size, egui::Sense::click());
                                egui::Image::new((tex.id(), *size)).paint_at(ui, rect);

                                if response.hovered() {
                                    ui.painter().rect_filled(
                                        rect,
                                        0.0,
                                        egui::Color32::from_rgba_unmultiplied(200, 200, 200, 40),
                                    );
                                }
                                if response.clicked() {
                                    self.selected = Some(p.clone());
                                    self.set_wallpaper(p);
                                }

                                if self.selected.as_deref() == Some(p.as_str()) {
                                    ui.painter().rect_stroke(
                                        rect,
                                        0.0,
                                        egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE),
                                    );
                                }
                            }
                        });
                        ui.add_space(5.0);
                        current_row.clear();
                        row_width = 0.0;
                    }

                    if !current_row.is_empty() {
                        row_width += padding;
                    }
                    row_width += img_size.x;
                    current_row.push((path, texture, img_size));
                }

                if !current_row.is_empty() {
                    let left_padding = (available_width - row_width) / 2.0;
                    ui.horizontal(|ui| {
                        ui.add_space(left_padding);
                        for (p, tex, size) in &current_row {
                            let (rect, response) =
                                ui.allocate_exact_size(*size, egui::Sense::click());
                            egui::Image::new((tex.id(), *size)).paint_at(ui, rect);

                            if response.hovered() {
                                ui.painter().rect_filled(
                                    rect,
                                    0.0,
                                    egui::Color32::from_rgba_unmultiplied(200, 200, 200, 40),
                                );
                            }
                            if response.clicked() {
                                self.selected = Some(p.clone());
                                self.set_wallpaper(p);
                            }
                            if self.selected.as_deref() == Some(p.as_str()) {
                                ui.painter().rect_stroke(
                                    rect,
                                    0.0,
                                    egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE),
                                );
                            }
                        }
                    });
                }
            });
        });

        // Error overlay (bottom)
        if let Some((msg, when)) = &self.last_error {
            if when.elapsed() < Duration::from_secs(2) {
                egui::TopBottomPanel::bottom("error_overlay")
                    .show_separator_line(false)
                    .show(ctx, |ui| {
                        ui.colored_label(egui::Color32::RED, msg);
                    });
            }
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let img_dir = dirs::home_dir().unwrap().join("Pictures/Wallpapers");

    let options = eframe::NativeOptions {
        viewport: egui::viewport::ViewportBuilder::default()
            .with_title("Hyprpaper GUI")
            .with_app_id("HyprpaperGUI"), 
        ..Default::default()
    };

    eframe::run_native(
        "Hyprpaper GUI",
        options,
        Box::new(move |_cc| Box::new(MyApp::new(img_dir))),
    )
}

