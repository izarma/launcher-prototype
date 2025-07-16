use eframe::App;
use poll_promise::Promise;
use reqwest::blocking::get;
use rfd::FileDialog;
use serde_json::Value;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use zip::ZipArchive;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Launhcer")
            //.with_inner_size([400.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_transparent(true)
            .with_always_on_top()
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("..\\assets\\icon.png")[..])
                    .expect("Failed to load icon"),
            ),
        persist_window: true,
        // default_theme: eframe::Theme::Dark,
        // mouse_passthrough: true, // Requires nightly
        ..Default::default()
    };
    eframe::run_native(
        "Launcher",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(Launcher::default()))
        }),
    )
}

struct Launcher {
    current_version: String,
    latest_version: String,
    path: String,
    release_notes: String,
    version_promise: Option<Promise<(String, String)>>,
}

impl Default for Launcher {
    fn default() -> Self {
        // Start fetching the latest version in a background thread
        let promise = Promise::spawn_thread("version_fetcher", fetch_latest_version_info);
        let (latest_version, release_notes) = fetch_latest_version_info();
        Self {
            current_version: "loading...".to_owned(),
            latest_version,
            release_notes,
            path: "E:\\Games\\Launcher".to_owned(),
            version_promise: Some(promise),
        }
    }
}
const URL : &str = "https://api.github.com/repos/makscee/arena-of-ideas/releases/latest";
impl App for Launcher {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("Current Version: {}", self.current_version));
            if self.current_version != self.latest_version {
                if ui.add(egui::Button::new("Update/Download")).clicked() {
                    download_update(self.path.clone());
                }
            }
            ui.label(format!("Latest Version: {}", self.latest_version));
            ui.label(format!("Installation Path: {}", self.path));
            if ui.add(egui::Button::new("Browse")).clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter("text", &["txt", "rs"])
                    .add_filter("rust", &["rs", "toml"])
                    .set_directory("/")
                    .pick_folder()
                {
                    self.path = path.to_string_lossy().to_string();
                }
            }
            ui.label(format!("Release Notes: \n {}", self.release_notes)); 
        });
    }
}

fn fetch_latest_version_info() -> (String, String) {
    // Create a client with a custom user-agent header (required by GitHub API)
    let client = reqwest::blocking::Client::builder()
        .user_agent("ArenaOfIdeasLauncher")
        .build()
        .expect("Failed to create HTTP client");

    // Send the GET request
    match client.get(URL).send() {
        Ok(response) => {
            // Check if we got a successful response
            if response.status().is_success() {
                // Parse the JSON response
                match response.json::<Value>() {
                    Ok(json) => {
                        // Extract the tag_name (version) and body (release notes)
                        let version = json["tag_name"]
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "Error: Missing version".to_string());

                        let notes = json["body"]
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "No release notes available".to_string());

                        let donwload_url = json["assets/url"]
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "No release notes available".to_string());

                        (version, notes)
                    }
                    Err(e) => (
                        "JSON Parse Error".to_string(),
                        format!("Error parsing response: {}", e),
                    ),
                }
            } else {
                (
                    "API Error".to_string(),
                    format!("API returned status: {}", response.status()),
                )
            }
        }
        Err(e) => (
            "Request Failed".to_string(),
            format!("Network error: {}", e),
        ),
    }
}

fn download_update(path: String) {
    let client = reqwest::blocking::Client::builder()
        .user_agent("ArenaOfIdeasLauncher")
        .build()
        .expect("Failed to create HTTP client");
    let url = "https://github.com/makscee/arena-of-ideas/releases/download/v1.8.2/arena-of-ideas-windows-v1.8.2.zip";
    let response = client.get(url).send().unwrap().bytes().unwrap();
    let file_path = Path::new(&path).join("arena-of-ideas.zip");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(&response).unwrap();
    println!("File saved to {}", path);
    let extract_to = Path::new(&path).join("arena-of-ideas");
    unzip_file(&file_path, &extract_to);
}

fn unzip_file(file_path: &Path, extract_to: &Path) {
    let zipfile = std::fs::File::open(file_path).unwrap();
    let mut archive = zip::ZipArchive::new(zipfile).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = Path::new(extract_to).join(file.mangled_name());

        if file.name().ends_with('/') {
            // Create directory
            fs::create_dir_all(&outpath).unwrap();
        } else {
            // Create parent directories
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }

            // Write file
            let mut outfile = File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    }
    fs::remove_file(file_path).unwrap();
    println!("Installed Successfully");
}
