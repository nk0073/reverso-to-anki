// 
//
//  TODO
//  - log to latest.log alongside with printing cus windows normies will want ta launch with .exe
//  instead of using console like normal people
//  - add pronounciations, though fetch them from somewhere else because reverso's voices are horrible
//  - maybe make it possible to provide custom css with a .css file
//
//

use std::io::Write;
use std::process::Child;
use std::process::Command;

use tempfile::NamedTempFile;
use thirtyfour::prelude::*;
use tokio::time::{Duration, sleep};

mod config;
mod cookies;
mod utils;
mod wordlist;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let cfg = config::get_config();
    let (mut geckodriver, driver) = init_driver(&cfg).await?;
    login(&driver, &cfg).await?;
    let option_node = get_words_node(&driver).await?;

    driver.quit().await?;
    geckodriver.kill().unwrap();

    if let Some(node) = option_node {
        let words = wordlist::scrape_node(&node);
        wordlist::update_list(&words, &cfg);
    } else {
        println!("You don't have any words saved! Nothing is changed.")
    }

    Ok(())
}

async fn init_driver(cfg: &config::Config) -> WebDriverResult<(Child, WebDriver)> {
    let driver_bytes = select_geckodriver();

    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(driver_bytes)?;

    #[cfg(unix)]
    {
        let mut perms: Permissions = temp_file.as_file().metadata()?.permissions();
        perms.set_mode(0o755); // +x
        temp_file.as_file().set_permissions(perms)?;
    }

    let path = temp_file.into_temp_path();
    let geckodriver = Command::new(&path)
        .arg("--port")
        .arg(cfg.port.to_string())
        .spawn()
        .unwrap();

    let mut caps = DesiredCapabilities::firefox();
    caps.unset_headless().unwrap();
    let http_driver = &format!("http://localhost:{}", cfg.port)[..];
    let driver = WebDriver::new(http_driver, caps).await.unwrap();

    Ok((geckodriver, driver))
}

async fn login(driver: &WebDriver, cfg: &config::Config) -> WebDriverResult<()> {
    let (mut cookies_file, cookies_file_created) = cookies::get_cookies_file();
    driver
        .get(format!(
            "https://www.reverso.net/favorites/{}",
            &cfg.language[..]
        ))
        .await?;

    let cookies_loaded_succssfully = cookies::load_cookies(&driver, &mut cookies_file).await;
    driver.refresh().await?;
    if !cookies_file_created && !cookies_loaded_succssfully {
        println!(
            "WARNING Cookies file was found, but wasn't loaded successfully. Erasing the cookies file..."
        );
        cookies_file.set_len(0)?;
    }

    match driver
        .query(By::ClassName("login-screen__button_primary"))
        .and_enabled()
        .wait(Duration::from_secs_f32(3.0), Duration::from_secs_f32(0.1))
        .first()
        .await
    {
        Ok(ele) => {
            println!("Found the button");
            ele.wait_until().clickable().await?;
            ele.click().await?;

            let interval = Duration::from_secs_f32(0.2);
            loop {
                // Wait until #login=success
                let url = driver.current_url().await?;
                let fragment = url.fragment();
                if fragment == Some("login=success") {
                    cookies::save_cookies(&driver, &mut cookies_file).await;
                    break;
                }
                sleep(interval).await;
            }
        }
        Err(_) => {} // The user is logged in
    };

    Ok(())
}

const CONTAINER_CLASS: &str = "list-favourites__container";
async fn get_words_node(driver: &WebDriver) -> WebDriverResult<Option<String>> {
    match driver
        .query(By::ClassName(CONTAINER_CLASS))
        .wait(Duration::from_secs_f32(3.0), Duration::from_secs_f32(0.2))
        .first()
        .await
    {
        Ok(val) => Ok(Some(val.inner_html().await?)),
        Err(_) => {
            // Container not found, the user has no favorites
            Ok(None)
        }
    }
}

// Embed the geckodriver in the binary
macro_rules! include_driver {
    ($name:expr) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/geckodrivers/", $name))
    };
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub fn select_geckodriver() -> &'static [u8] {
    include_driver!("geckodriver_win64.exe")
}

#[cfg(all(target_os = "windows", target_arch = "x86"))]
pub fn select_geckodriver() -> &'static [u8] {
    include_driver!("geckodriver_win32.exe")
}

#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
pub fn select_geckodriver() -> &'static [u8] {
    include_driver!("geckodriver_win_aarch64.exe")
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn select_geckodriver() -> &'static [u8] {
    include_driver!("geckodriver_linux64")
}

#[cfg(all(target_os = "linux", target_arch = "x86"))]
pub fn select_geckodriver() -> &'static [u8] {
    include_driver!("geckodriver_linux32")
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub fn select_geckodriver() -> &'static [u8] {
    include_driver!("geckodriver_linux_aarch64")
}

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub fn select_geckodriver() -> &'static [u8] {
    include_driver!("geckodriver_mac")
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn select_geckodriver() -> &'static [u8] {
    include_driver!("geckodriver_mac_aarch64")
}

#[cfg(not(any(
    all(target_os = "windows", target_arch = "x86_64"),
    all(target_os = "windows", target_arch = "x86"),
    all(target_os = "windows", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "x86"),
    all(target_os = "linux", target_arch = "aarch64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64")
)))]
compile_error!("Unsupported platform: no geckodriver embedded for this OS/arch.");

