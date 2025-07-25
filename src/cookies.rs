use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use thirtyfour::{Cookie, WebDriver};

use serde_json;
use crate::utils;

// Returns the file and the bool which is true if the file was created, false if it existed before
const COOKIES_FILE_NAME: &str = "cookies.json";
pub fn get_cookies_file() -> (File, bool) {
    let path = utils::get_path(COOKIES_FILE_NAME);
    if !Path::new(&path).exists() {
        println!("Creating cookies file");
        let file = File::create(&path).unwrap();
        println!("Cookies file created succesfully");

        return (file, true);
    } else {
        println!("Cookies file found");
        let file = File::open(&path).unwrap();

        return (file, false);
    }
}

// The function returns false if it's failed, no need for the Error type
// because it's unrecoverable
pub async fn load_cookies(driver: &WebDriver, cookie_file: &mut File) -> bool {
    let mut file_out = String::new();
    let bytes_read = cookie_file.read_to_string(&mut file_out);
    if bytes_read.is_err() || bytes_read.ok() == Some(0) {
        return false;
    }

    let cookies: Vec<Cookie> = match serde_json::from_str(&file_out) {
        Ok(val) => val,
        Err(_) => {
            return false;
        }
    };

    for cookie in cookies {
        match driver.add_cookie(cookie).await {
            Ok(_) => {}
            Err(_) => {
                driver.delete_all_cookies().await.unwrap();
                return false;
            }
        };
    }

    true
}

pub async fn save_cookies(driver: &WebDriver, cookie_file: &mut File) {
    let cookies = driver.get_all_cookies().await.unwrap();
    let cookies_json = serde_json::to_string(&cookies).unwrap();
    match cookie_file.write(cookies_json.as_bytes()) {
        Ok(_) => {
            println!("Wrote cookies to the file");
        }
        Err(_) => {
            eprintln!("WARNING Failed to write the cookie file");
        }
    }
}
