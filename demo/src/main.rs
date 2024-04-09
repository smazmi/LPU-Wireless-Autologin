use std::{
    env, io,
    process::{self, Command},
};
fn main() {
    // list_account_ids();
    println!("{}", check_lpu_wifi());
}

fn show_version() {
    println!("LPU WiFi Manager 0.1.0r");
}

fn show_help() {
    println!("Usage: llogin [OPTIONS] [ACCOUNT_ID]");
    println!("Manage and log in to multiple LPU WiFi accounts.");
    println!(" ");
    println!("Options:");
    println!(" --help      Show this help message and exit.");
    println!(" --version   Show version information and exit.");
    println!(" --list      List all stored account IDs.");
}

fn prompt_for_account_id() {
    let mut account_id = String::new();
    println!("Enter the account ID or Name: ");
    io::stdin()
        .read_line(&mut account_id)
        .expect("Failed to read line");
    println!("{}", account_id);
}

fn list_account_ids() {
    // println!("Stored account IDs: ");
    // for (key, value) in env::vars() {
    //     println!("key: {}, value: {}", key, value);
    // }
    // println!("{:?}",option_env!("LPU_USERNAME"));
}

fn check_lpu_wifi() -> bool {
    let output = Command::new("nmcli")
        .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()
        .expect("Failed to execute nmcli");
    // String conversion
    let output_str = String::from_utf8_lossy(&output.stdout);
    //Check if connected to LPU WiFi
    if output_str.contains("yes:LPU") {
        return true;
    } else {
        return false;
    }
}
