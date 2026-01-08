use anyhow::{Context, Result};
use grammers_client::Client;
use grammers_client::SignInError;
use grammers_client::types::InputMessage;
use grammers_session::Session;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tracing::{info, error};

// Include Slint UI
slint::include_modules!();

// Use the generated FileEntry from Slint
use slint_generatedAppWindow::FileEntry as SlintFileEntry;

// Constants
const DB_FILE: &str = "telegram_cloud.json";

/// File record structure for JSON storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileRecord {
    filename: String,
    file_id: String,
    upload_date: String,
    file_size: u64,
}

/// Database management using JSON file storage
struct Database {
    file_path: PathBuf,
    records: Arc<Mutex<Vec<FileRecord>>>,
}

impl Database {
    async fn new(db_path: &str) -> Result<Self> {
        let file_path = PathBuf::from(db_path);
        let records = if file_path.exists() {
            let content = tokio::fs::read_to_string(&file_path).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };
        
        Ok(Self {
            file_path,
            records: Arc::new(Mutex::new(records)),
        })
    }
    
    async fn save(&self) -> Result<()> {
        let records = self.records.lock().unwrap().clone();
        let json = serde_json::to_string_pretty(&records)?;
        
        let mut file = tokio::fs::File::create(&self.file_path).await?;
        file.write_all(json.as_bytes()).await?;
        
        Ok(())
    }
    
    async fn insert_file(&self, filename: &str, file_id: &str, file_size: u64) -> Result<()> {
        let upload_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let record = FileRecord {
            filename: filename.to_string(),
            file_id: file_id.to_string(),
            upload_date,
            file_size,
        };
        
        self.records.lock().unwrap().push(record);
        self.save().await?;
        
        Ok(())
    }
    
    fn get_all_files(&self) -> Result<Vec<SlintFileEntry>> {
        let records = self.records.lock().unwrap();
        let mut files: Vec<SlintFileEntry> = records
            .iter()
            .map(|r| SlintFileEntry {
                filename: r.filename.clone().into(),
                file_id: r.file_id.clone().into(),
                upload_date: r.upload_date.clone().into(),
                size: format_size(r.file_size).into(),
            })
            .collect();
        
        files.reverse();
        Ok(files)
    }
}

/// Format file size
fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

/// Upload file to Telegram
async fn upload_file_to_telegram(
    client: &Client,
    file_path: &Path,
    ui_handle: slint::Weak<AppWindow>,
) -> Result<String> {
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .context("Invalid filename")?;
    
    info!("Starting upload for: {}", filename);
    
    let metadata = tokio::fs::metadata(file_path).await?;
    let file_size = metadata.len();
    
    info!("File size: {} bytes", file_size);
    
    // Update UI
    let ui_clone = ui_handle.clone();
    if let Some(ui) = ui_clone.upgrade() {
        let filename_clone = filename.to_string();
        let _result = ui.invoke_from_event_loop(move || {
            ui.set_status_text(format!("Uploading {}...", filename_clone).into());
            ui.set_upload_progress(0.1);
        });
    }
    
    // Upload file
    let uploaded = client.upload_file(file_path).await?;
    
    // Update progress
    let ui_clone = ui_handle.clone();
    if let Some(ui) = ui_clone.upgrade() {
        let _result = ui.invoke_from_event_loop(move || {
            ui.set_upload_progress(0.8);
        });
    }
    
    // Send to Saved Messages - use Chat reference
    info!("Sending file to Saved Messages...");
    let me = client.get_me().await?;
    let chat = client.resolve_username("me").await?
        .ok_or_else(|| anyhow::anyhow!("Failed to resolve self"))?;
    
    // Create input message
    let input_msg = InputMessage::default().document(uploaded);
    client.send_message(chat, input_msg).await?;
    
    // Final progress update
    let ui_clone = ui_handle.clone();
    if let Some(ui) = ui_clone.upgrade() {
        let _result = ui.invoke_from_event_loop(move || {
            ui.set_upload_progress(1.0);
        });
    }
    
    info!("Upload completed!");
    Ok(format!("tg_file_{}", filename))
}

/// Initialize Telegram client
async fn init_telegram_client(
    api_id: i32,
    api_hash: &str,
    session_file: &str,
) -> Result<Client> {
    info!("Initializing Telegram client...");
    
    // Load session
    let session = Session::load_file_or_create(session_file)?;
    
    // Connect to Telegram
    let client = Client::connect(grammers_client::Config {
        session,
        api_id,
        api_hash: api_hash.to_string(),
        params: Default::default(),
    }).await?;
    
    info!("Client connected");
    Ok(client)
}

/// Handle phone authentication
async fn authenticate_with_phone(
    client: &Client,
    phone: &str,
    api_hash: &str,
) -> Result<()> {
    info!("Starting authentication with phone: {}", phone);
    
    if !client.is_authorized().await? {
        info!("Not authorized, requesting code...");
        
        // Request login code
        let token = client.request_login_code(phone, api_hash).await?;
        
        // Read code from stdin
        println!("Enter the code you received:");
        let mut code = String::new();
        std::io::stdin().read_line(&mut code)?;
        let code = code.trim();
        
        // Sign in
        match client.sign_in(&token, code).await {
            Ok(_) => {
                info!("Successfully signed in!");
            }
            Err(SignInError::PasswordRequired(password_token)) => {
                println!("Enter your 2FA password:");
                let mut password = String::new();
                std::io::stdin().read_line(&mut password)?;
                let password = password.trim();
                
                client.check_password(password_token, password).await?;
                info!("Successfully signed in with 2FA!");
            }
            Err(e) => return Err(e.into()),
        }
    } else {
        info!("Already authorized");
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load environment variables
    dotenv::dotenv().ok();
    
    let api_id = std::env::var("API_ID")
        .context("API_ID not found in .env")?
        .parse::<i32>()
        .context("API_ID must be a number")?;
    
    let api_hash = std::env::var("API_HASH")
        .context("API_HASH not found in .env")?;
    
    let session_name = std::env::var("SESSION_NAME")
        .unwrap_or_else(|_| "telegram_cloud.session".to_string());
    
    info!("Starting Telegram Cloud Storage application");
    
    // Initialize database
    let db = Arc::new(Database::new(DB_FILE).await?);
    
    // Create UI
    let ui = AppWindow::new()?;
    let ui_weak = ui.as_weak();
    
    // State management
    let client: Arc<Mutex<Option<Client>>> = Arc::new(Mutex::new(None));
    let selected_file: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));
    
    // Select file callback
    {
        let selected_file = selected_file.clone();
        let ui_weak = ui_weak.clone();
        
        ui.on_select_file(move || {
            let file = rfd::FileDialog::new().pick_file();
            
            if let Some(path) = file {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                *selected_file.lock().unwrap() = Some(path);
                
                let ui_clone = ui_weak.clone();
                if let Some(ui) = ui_clone.upgrade() {
                    ui.set_selected_file(filename.as_str().into());
                    ui.set_status_text("File selected. Ready to upload.".into());
                }
            }
        });
    }
    
    // Authenticate callback
    {
        let client = client.clone();
        let ui_weak = ui_weak.clone();
        
        ui.on_authenticate(move |phone| {
            let phone = phone.to_string();
            let client = client.clone();
            let ui_weak = ui_weak.clone();
            let api_id = api_id;
            let api_hash = api_hash.clone();
            let session_name = session_name.clone();
            
            tokio::spawn(async move {
                let ui_clone = ui_weak.clone();
                if let Some(ui) = ui_clone.upgrade() {
                    ui.set_status_text("Connecting to Telegram...".into());
                }
                
                match init_telegram_client(api_id, &api_hash, &session_name).await {
                    Ok(tg_client) => {
                        match authenticate_with_phone(&tg_client, &phone, &api_hash).await {
                            Ok(_) => {
                                *client.lock().unwrap() = Some(tg_client);
                                
                                let ui_clone = ui_weak.clone();
                                if let Some(ui) = ui_clone.upgrade() {
                                    ui.set_is_authenticated(true);
                                    ui.set_status_text("Successfully authenticated!".into());
                                }
                            }
                            Err(e) => {
                                error!("Authentication failed: {:?}", e);
                                let ui_clone = ui_weak.clone();
                                if let Some(ui) = ui_clone.upgrade() {
                                    ui.set_status_text(format!("Auth failed: {}", e).into());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to connect: {:?}", e);
                        let ui_clone = ui_weak.clone();
                        if let Some(ui) = ui_clone.upgrade() {
                            ui.set_status_text(format!("Connection failed: {}", e).into());
                        }
                    }
                }
            });
        });
    }
    
    // Upload file callback
    {
        let selected_file = selected_file.clone();
        let client = client.clone();
        let db = db.clone();
        let ui_weak = ui_weak.clone();
        
        ui.on_upload_file(move || {
            let file_path = selected_file.lock().unwrap().clone();
            
            if let Some(path) = file_path {
                let client = client.clone();
                let db = db.clone();
                let ui_weak = ui_weak.clone();
                
                tokio::spawn(async move {
                    let ui_clone = ui_weak.clone();
                    if let Some(ui) = ui_clone.upgrade() {
                        ui.set_is_uploading(true);
                        ui.set_upload_progress(0.0);
                        ui.set_status_text("Starting upload...".into());
                    }
                    
                    // Clone client outside the lock to avoid holding it across await
                    let tg_client = {
                        let client_guard = client.lock().unwrap();
                        client_guard.clone()
                    };
                    
                    if let Some(tg_client) = tg_client {
                        let file_size = tokio::fs::metadata(&path).await
                            .map(|m| m.len())
                            .unwrap_or(0);
                        
                        match upload_file_to_telegram(&tg_client, &path, ui_weak.clone()).await {
                            Ok(file_id) => {
                                let filename = path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Unknown");
                                
                                if let Err(e) = db.insert_file(filename, &file_id, file_size).await {
                                    error!("Failed to save to database: {:?}", e);
                                }
                                
                                let ui_clone = ui_weak.clone();
                                if let Some(ui) = ui_clone.upgrade() {
                                    ui.set_status_text("Upload successful!".into());
                                    ui.set_selected_file("No file selected".into());
                                }
                            }
                            Err(e) => {
                                error!("Upload failed: {:?}", e);
                                let ui_clone = ui_weak.clone();
                                if let Some(ui) = ui_clone.upgrade() {
                                    ui.set_status_text(format!("Upload failed: {}", e).into());
                                }
                            }
                        }
                    }
                    
                    let ui_clone = ui_weak.clone();
                    if let Some(ui) = ui_clone.upgrade() {
                        ui.set_is_uploading(false);
                        ui.set_upload_progress(0.0);
                    }
                });
            }
        });
    }
    
    // Refresh files callback
    {
        let db = db.clone();
        let ui_weak = ui_weak.clone();
        
        ui.on_refresh_files(move || {
            let db = db.clone();
            let ui_weak = ui_weak.clone();
            
            tokio::spawn(async move {
                match db.get_all_files() {
                    Ok(files) => {
                        let ui_clone = ui_weak.clone();
                        if let Some(ui) = ui_clone.upgrade() {
                            let files_rc = std::rc::Rc::new(slint::VecModel::from(files));
                            ui.set_uploaded_files(files_rc.into());
                        }
                    }
                    Err(e) => {
                        error!("Failed to load files: {:?}", e);
                    }
                }
            });
        });
    }
    
    ui.run()?;
    
    Ok(())
}
