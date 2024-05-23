mod app_lib;

use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};
use std::io;
use std::time::Instant;
use app_lib::gui::AppState;
use app_lib::file_processor::traverse_directory;

fn run_conversion(src_dir: &Path, dest_dir: &Path, log: Arc<RwLock<Vec<String>>>, count: Arc<RwLock<u32>>, elapsed_time: Arc<RwLock<String>>, batch_size: usize, in_ui_mode: bool) -> io::Result<()> {
    let start = Instant::now();
    let result = traverse_directory(src_dir, dest_dir, PathBuf::new(), log.clone(), count.clone(), batch_size, in_ui_mode);
    let elapsed = start.elapsed();
    let msg = format!("Elapsed time: {:.2?}", elapsed);
    if !in_ui_mode {
        println!("{}", msg);
    }
    {
        let mut log = log.write().unwrap();
        log.push(msg.clone());
        if log.len() > batch_size {
            log.remove(0);
        }
    }
    {
        let mut elapsed_time = elapsed_time.write().unwrap();
        *elapsed_time = msg;
    }
    result
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 {
        let src_dir = Path::new(&args[1]);
        let dest_dir = Path::new(&args[2]);
        let log = Arc::new(RwLock::new(Vec::new()));
        let file_count = Arc::new(RwLock::new(0));
        let elapsed_time = Arc::new(RwLock::new(String::new()));
        let batch_size = 100;  // Batch size for log updates
        return run_conversion(src_dir, dest_dir, log, file_count, elapsed_time, batch_size, false);
    }

    let app = AppState::default();
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(960.0, 300.0)),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), native_options);

}
