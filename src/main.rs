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
// use rfd::FileDialog;
use md5;
use walkdir::WalkDir;
use serde_json;

struct HppG {
    img_dir: PathBuf,
    folder_textures: HashMap<String, Vec<(String, egui::TextureHandle)>>,
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

impl HppG {
    fn new(cc: &eframe::CreationContext<'_>,img_dir: PathBuf) -> Self {
        cc.egui_ctx.set_pixels_per_point(1.2); // set default UI zoom 
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default()).unwrap();
        watcher.watch(&img_dir, RecursiveMode::Recursive).unwrap();
    
        let monitors = Self::get_monitors();
        let selected_monitor = monitors.get(0).cloned();

        Self {
            img_dir,
            folder_textures: HashMap::new(),
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
        self.folder_textures.clear();

        let thumb_dir = dirs::home_dir().unwrap().join(".cache/thumbnails/large");
        let _ = fs::create_dir_all(&thumb_dir);

        for entry in WalkDir::new(&self.img_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path().to_path_buf();
            if !path.is_file() {
                continue;
            }

            // file URI → md5 digest
            let uri = format!("file://{}", path.to_string_lossy());
            let digest = format!("{:x}", md5::compute(uri.as_bytes()));
            let thumb_path = thumb_dir.join(format!("{}.png", digest));

            let thumb_img = if thumb_path.exists() {
                image::open(&thumb_path).ok()
            } else {
                None
            };

            if let Some(img) = thumb_img {
                let rgba = img.to_rgba8();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [img.width() as usize, img.height() as usize],
                    rgba.as_flat_samples().as_slice(),
                );
                let texture = ctx.load_texture(path.to_string_lossy(), color_image, Default::default());

                let folder_name = if path.parent() == Some(&self.img_dir) {
                    // File is directly under /Wallpapers
                    "Root".to_string()
                } else {
                    path.parent()
                        .and_then(|p| p.file_name())
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                };

                self.folder_textures
                    .entry(folder_name)
                    .or_default()
                    .push((path.to_string_lossy().to_string(), texture));
            }
        }
    }


    fn set_wallpaper(&mut self, path: &str) {
        if let Some(monitor) = &self.selected_monitor {
            let abs = std::fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));

            let preload = Command::new("hyprctl")
                .args(&["hyprpaper", "preload", &abs.to_string_lossy()])
                .output();

            if preload.as_ref().map(|o| o.status.success()).unwrap_or(false) {
                let wallpaper = Command::new("hyprctl")
                    .args(&["hyprpaper", "wallpaper", &format!("{},{}", monitor, abs.display())])
                    .output();

                if wallpaper.as_ref().map(|o| o.status.success()).unwrap_or(false) {
                    let _ = Command::new("hyprctl")
                        .args(&["hyprpaper", "reload"])
                        .output();
                } else {
                    self.last_error = Some(("Failed to apply wallpaper".into(), Instant::now()));
                }
            } else {
                self.last_error = Some(("Failed to preload wallpaper".into(), Instant::now()));
            }
        } else {
            self.last_error = Some(("No monitor selected".into(), Instant::now()));
        }
    }
}

impl eframe::App for HppG {
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
            ui.add_space(10.0); // top padding

            ui.horizontal(|ui| {
                ui.add_space(10.0); // left padding

                // Zoom controls
                ui.label("Zoom:");
                if ui.button("➕").clicked() {
                    self.thumbnail_scale *= 1.2;
                }
                if ui.button("➖").clicked() {
                    self.thumbnail_scale /= 1.2;
                }
                ui.add(egui::Slider::new(&mut self.thumbnail_scale, 0.5..=3.0));

                ui.add_space(20.0);

                // Push monitor selector to the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(10.0); // right padding
                        let flashing = self.monitor_warning_until
                            .map(|t| Instant::now() < t)
                            .unwrap_or(false);

                        let _combo_visuals = if flashing {
                            let t = (Instant::now().elapsed().as_millis() as f32 / 200.0).sin();
                            egui::Color32::from_rgba_unmultiplied(255, 0, 0, (t * 127.0 + 128.0) as u8)
                        } else {
                            ui.visuals().widgets.inactive.bg_fill
                        };

                        egui::ComboBox::from_id_source("monitor_selector")
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
                        ui.label("Monitor:");
                    });
                });
                ui.add_space(10.0); // right padding
            });

            ui.add_space(10.0); // bottom padding
        });


        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let target_height = 128.0 * self.thumbnail_scale;
                let padding = 4.0;
                let available_width = ui.available_width();

                let folders: Vec<(String, Vec<(String, egui::TextureHandle)>)> =
                    self.folder_textures
                        .iter()
                        .map(|(folder, textures)| {
                            (
                                folder.clone(),
                                textures
                                    .iter()
                                    .map(|(p, tex)| (p.clone(), tex.clone()))
                                    .collect(),
                            )
                        })
                        .collect();

                for (_folder, textures) in folders {
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
                    // add space between different folders (SKIP AT THE MOMENT)
                    ui.add_space(50.0);
                    //ui.add_space(20.0);
                    //ui.with_layout(
                    //   egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    //    |ui| {
                    //        ui.label(
                    //            egui::RichText::new(_folder.clone())
                    //                .heading()
                    //                .color(egui::Color32::from_rgba_unmultiplied(48, 200, 255, 120)) // alpha same from 255
                    //                .strong(),
                    //        );
                    //    },
                    //);
                    // ui.add_space(10.0);
                }
            });
        });

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
        Box::new(move |cc| Box::new(HppG::new(cc, img_dir.clone()))),
    )
}
