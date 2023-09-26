use std::env;
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use reqwest;
use hostname;
use pnet::datalink;

#[derive(Serialize, Deserialize)]
struct SSHKeyReport {
    vm_name: String,
    vm_uuid: String,
    ip_address: Option<String>,
    keys: Vec<String>,
}

fn get_primary_ip() -> Option<String> {
    for iface in datalink::interfaces() {
        // Check if the interface is not a loopback
        if !iface.is_loopback() {
            for ip in iface.ips {
                if ip.is_ipv4() {
                    return Some(ip.ip().to_string());
                }
            }
        }
    }
    None
}

fn read_ssh_keys(path: &Path) -> Option<Vec<String>> {
    if let Ok(content) = fs::read_to_string(path) {
        Some(content.lines().map(|s| s.to_string()).collect())
    } else {
        None
    }
}

fn get_vm_uuid() -> Result<String, std::io::Error> {
    fs::read_to_string("/sys/class/dmi/id/product_uuid")
}

async fn send_to_server(report: SSHKeyReport, server_url: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    client.post(server_url).json(&report).send().await?;
    Ok(())
}

// This function reads the /etc/passwd file and returns a list of home directories
fn get_user_home_dirs() -> Vec<String> {
    let content = fs::read_to_string("/etc/passwd").unwrap_or_default();
    content.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 5 {
                Some(parts[5].to_string())
            } else {
                None
            }
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vm_name = match hostname::get().map(|os_str| os_str.into_string()) {
        Ok(Ok(string)) => string,
        Ok(Err(_)) | Err(_) => "unknown".to_string(),
    };
    
    let ip_address = get_primary_ip();
    let vm_uuid_result = get_vm_uuid();
    let vm_uuid = match vm_uuid_result {
        Ok(uuid) => uuid,
        Err(e) => {
            eprintln!("Error getting VM UUID: {}", e);
            "unknown".to_string() // or handle this error in another way
        }
    };
    // Fetch server_url from environment variable
    let server_url = env::var("SERVER_URL").unwrap_or_else(|_| "http://10.236.173.129.nip.io:8000/".to_string());

    for home_dir in get_user_home_dirs() {
        let key_path = Path::new(&home_dir).join(".ssh/authorized_keys");
        if let Some(keys) = read_ssh_keys(&key_path) {
            let report = SSHKeyReport {
                vm_name: vm_name.clone(),
                ip_address: ip_address.clone(),
                vm_uuid: vm_uuid.clone(),
                keys,
            };
            send_to_server(report, &server_url).await?;
        }
    }

    Ok(())
}
