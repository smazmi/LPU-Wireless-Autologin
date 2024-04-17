use dirs_next::home_dir;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process;
use std::{env, process::Command};

fn show_help() {
    println!(
        "Usage: {} [OPTION] [ACCOUNT_ID]",
        env::args().next().unwrap()
    );
    println!("Manage and log in to multiple LPU WiFi accounts.");
    println!();
    println!("Options:");
    println!(" --help or -h       Show this help message and exit.");
    println!(" --version or -v    Show version information and exit.");
    println!(" --list or -l       List all stored account IDs.");
    println!(" --account or -a    Followed by the account ID you want to login as.");
}

fn show_version() {
    println!("LPU WiFi Manager 1.2");
}

fn prompt_for_account_id() -> String {
    print!("Enter the account ID or Name: ");
    io::stdout().flush().unwrap();
    let mut account_id = String::new();
    io::stdin().read_line(&mut account_id).unwrap();
    account_id.trim().to_string()
}

fn list_account_ids() {
    println!("Stored account IDs:");
    let mut found = false;
    for (key, _value) in env::vars() {
        if key.starts_with("LPU_USERNAME_") {
            if let Some(account_id) = key.split('_').last() {
                println!("{}", account_id);
                found = true;
            }
        }
    }

    if !found {
        println!("No stored account IDs found.");
    }
}

fn check_lpu_wifi() -> bool {
    if let Ok(output) = std::process::Command::new("nmcli")
        .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()
    {
        if let Ok(output_string) = String::from_utf8(output.stdout) {
            return output_string
                .lines()
                .any(|line| line.starts_with("yes:LPU"));
        }
    }
    false
}

fn store_lpu_credentials() {
    print!("Enter a unique identifier for this account: ");
    io::stdout().flush().unwrap();
    let account_id = prompt_for_account_id();

    let username_var = format!("LPU_USERNAME_{}", account_id);
    let password_var = format!("LPU_PASSWORD_{}", account_id);

    if env::var(&username_var).is_ok() || env::var(&password_var).is_ok() {
        println!("Credentials already exist for account ID '{}'.", account_id);
        return;
    }

    print!("Enter your LPU username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();

    print!("Enter your LPU password: ");
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();

    env::set_var(&username_var, username.trim());
    env::set_var(&password_var, password.trim());

    // Write credentials to file
    let home_dir = home_dir().expect("Failed to get home directory");
    let lpu_creds_path = home_dir.join(".lpu_creds");
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(lpu_creds_path)
        .unwrap();

    if let Err(e) = writeln!(file, "export {}=\"{}\"", username_var, username.trim()) {
        eprintln!("Couldn't write to file: {}", e);
    }

    if let Err(e) = writeln!(file, "export {}=\"{}\"", password_var, password.trim()) {
        eprintln!("Couldn't write to file: {}", e);
    }

    // Update shell configuration file
    let shell_config = match env::var("SHELL") {
        Ok(val) => {
            if val.contains("bash") {
                home_dir.join(".bashrc")
            } else if val.contains("zsh") {
                home_dir.join(".config/zsh/.zshrc")
            } else if val.contains("fish") {
                home_dir.join(".config/fish/config.fish")
            } else {
                println!("Unsupported shell. Please manually set the environment variables.");
                return;
            }
        }
        Err(_) => {
            println!("Couldn't determine shell.");
            return;
        }
    };

    let file_path = Path::new(&shell_config);
    let file = File::open(&file_path).unwrap();
    let reader = BufReader::new(file);

    // Check if the source command already exists in the shell configuration file
    println!("LPU username and password have been stored securely. Reload the current shell or open a new one to use it :)");
    if reader
        .lines()
        .any(|line| line.unwrap() == "source ~/.lpu_creds")
    {
        return;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&shell_config)
        .unwrap();

    if let Err(e) = writeln!(file, "source ~/.lpu_creds") {
        eprintln!("Couldn't write to file: {}", e);
    }
}

fn perform_lpu_login(account_id: &str) {
    let username_var = format!("LPU_USERNAME_{}", account_id);
    let password_var = format!("LPU_PASSWORD_{}", account_id);

    let username = env::var(&username_var).unwrap_or(String::new());
    let password = env::var(&password_var).unwrap_or(String::new());

    if !username.is_empty() && !password.is_empty() {
        let data = format!(
            "mode=191&username={}%40lpu.com&password={}",
            username, password
        );
        let output = Command::new("curl")
            .arg("-s")
            .arg("-d")
            .arg(&data)
            .arg("--compressed")
            .arg("--insecure")
            .arg("https://10.10.0.1/24online/servlet/E24onlineHTTPClient")
            .output()
            .expect("Failed to execute curl command.");
        let res = String::from_utf8_lossy(&output.stdout);
        if res.contains("To start surfing") {
            println!("Login successful for account {}.", account_id);
        } else {
            println!("Login failed for account {}.", account_id);
        }
    } else {
        println!(
            "LPU username or password not set for account {}.",
            account_id
        );
        store_lpu_credentials();
    }
}

fn main() {
    if !check_lpu_wifi() {
        println!("Not connected to LPU WiFi. Exiting.");
        process::exit(1);
    }

    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("No account ID provided.");
        let account_id = prompt_for_account_id();
        perform_lpu_login(&account_id);
    } else {
        match args[1].as_str() {
            "--help" | "-h" => {
                show_help();
            }
            "--version" | "-v" => {
                show_version();
            }
            "--account" | "-a" => {
                if args.len() != 3 {
                    println!("Error: Please provide an account ID.");
                    process::exit(1);
                } else {
                    perform_lpu_login(&args[2]);
                }
            }
            "--list" | "-l" => {
                list_account_ids();
            }
            _ => {
                println!("Error: Unknown option. Use --help for usage information.");
                process::exit(1);
            }
        }
    }
}

