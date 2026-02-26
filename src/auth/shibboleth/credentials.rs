use serde::Serialize;
use serde_json::from_reader;
use std::fs::{remove_file, File};
use std::io::{stdin, BufReader, Write};
use std::path::PathBuf;

#[derive(Serialize, Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    fn ask_credentials(path_to_file: PathBuf) -> Credentials {
        print!("Enter ilias email: ");
        std::io::stdout().flush().unwrap();

        let mut username = String::new();
        // `read_line` returns `Result` of bytes read
        stdin().read_line(&mut username).unwrap();
        username = username.trim().to_string();

        print!("Enter ilias password: ");
        std::io::stdout().flush().unwrap();

        let mut password = String::new();
        // `read_line` returns `Result` of bytes read
        stdin().read_line(&mut password).unwrap();
        password = password.trim().to_string();

        let credentials = Credentials { username, password };

        // Write to credentials file
        let mut file = File::create_new(path_to_file.clone()).unwrap();
        file.set_len(0).unwrap();
        let json_str = serde_json::to_string(&credentials);
        file.write_all(json_str.unwrap().as_bytes()).unwrap();

        credentials
    }

    pub fn new(path_to_file: PathBuf) -> Credentials {
        // open file as json
        let file = File::open(path_to_file.clone());
        if file.is_err() {
            let message = format!("credentials file not found: {:?}", path_to_file.clone());
            println!("{}", message);

            return Self::ask_credentials(path_to_file);
        }
        let file = file.unwrap();

        let reader = BufReader::new(file);
        let json: serde_json::Value = from_reader(reader).unwrap();

        if json.get("username").is_none() {
            remove_file(path_to_file.clone()).unwrap();
            println!("username not found in credentials file");

            return Self::ask_credentials(path_to_file);
        }
        if json.get("password").is_none() {
            remove_file(path_to_file.clone()).unwrap();
            println!("password not found in credentials file");

            return Self::ask_credentials(path_to_file);
        }

        // get username and password from json
        let username = json["username"].as_str().unwrap().to_string();
        let password = json["password"].as_str().unwrap().to_string();

        Credentials { username, password }
    }
}
