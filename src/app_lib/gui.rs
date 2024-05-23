use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use eframe::egui;
use eframe::epi;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

use crate::run_conversion;

#[derive(Default, Serialize, Deserialize)]
struct Config {
    source_directory: String,
    destination_directory: String,
}

impl Config {
    fn load() -> Self {
        if let Ok(config_data) = fs::read_to_string("config.json") {
            if let Ok(config) = serde_json::from_str(&config_data) {
                return config;
            }
        }
        Self::default()
    }

    fn save(&self) -> io::Result<()> {
        let config_data = serde_json::to_string(self)?;
        let mut file = fs::File::create("config.json")?;
        file.write_all(config_data.as_bytes())?;
        Ok(())
    }
}

pub struct AppState {
    pub source_directory: String,
    pub destination_directory: String,
    pub log: Arc<RwLock<Vec<String>>>,
    pub file_count: Arc<RwLock<u32>>,
    pub elapsed_time: Arc<RwLock<String>>,
    pub is_running: bool,
    pub in_ui_mode: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let config = Config::load();
        Self {
            source_directory: config.source_directory,
            destination_directory: config.destination_directory,
            log: Arc::new(RwLock::new(Vec::new())),
            file_count: Arc::new(RwLock::new(0)),
            elapsed_time: Arc::new(RwLock::new(String::new())),
            is_running: false,
            in_ui_mode: true,
        }
    }
}

impl epi::App for AppState {
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Source Directory:");
                ui.text_edit_singleline(&mut self.source_directory);
                if ui.button("Select").clicked() {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        self.source_directory = path.display().to_string();
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Destination Directory:");
                ui.text_edit_singleline(&mut self.destination_directory);
                if ui.button("Select").clicked() {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        self.destination_directory = path.display().to_string();
                    }
                }
            });
            if ui.button("Start Conversion").clicked() {
                if !self.source_directory.is_empty() && !self.destination_directory.is_empty() {
                    self.is_running = true;
                    let src_dir = PathBuf::from(&self.source_directory);
                    let dest_dir = PathBuf::from(&self.destination_directory);
                    let log = self.log.clone();
                    let count = self.file_count.clone();
                    let elapsed_time = self.elapsed_time.clone();
                    let batch_size = 100;  // Batch size for log updates
                    let in_ui_mode = self.in_ui_mode;

                    // Save config
                    let config = Config {
                        source_directory: self.source_directory.clone(),
                        destination_directory: self.destination_directory.clone(),
                    };
                    let _ = config.save();

                    std::thread::spawn(move || {
                        let _ = run_conversion(&src_dir, &dest_dir, log, count, elapsed_time, batch_size, in_ui_mode);
                    });
                }
            }
            ui.add_space(10.0);
            ui.label(format!("Files processed: {}", *self.file_count.read().unwrap()));
            ui.label(format!("{}", *self.elapsed_time.read().unwrap()));
            ui.label("Log Output:");
            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                let log = self.log.read().unwrap();
                for entry in log.iter() {
                    ui.label(entry);
                }
                ui.scroll_to_cursor(egui::Align::BOTTOM);
            });
            if self.is_running {
                ui.ctx().request_repaint();
            }
        });
    }

    fn name(&self) -> &str {
        "MSA to ST Converter"
    }
}
