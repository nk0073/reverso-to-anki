use thirtyfour::prelude::*;
use tokio::time::{Duration, sleep};

mod cookies;
mod wordlist;
mod utils;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::firefox();
    caps.unset_headless().unwrap();
    let http_driver = "http://localhost:4444";
    let driver = match WebDriver::new(http_driver, caps).await {
        Ok(val) => val,
        Err(err) => {
            panic!("{}", err.to_string());
        }
    };

    login(&driver).await?;
    let option_node = get_words_node(&driver).await?;
    driver.quit().await?;
    if let Some(node) = option_node {
        let words = wordlist::scrape_node(&node);
        wordlist::update_list(&words);
    } else {
        println!("You don't have any words saved! Nothing is changed.")
    }

    Ok(())
}

async fn login(driver: &WebDriver) -> WebDriverResult<()> {
    let (mut cookies_file, cookies_file_created) = cookies::get_cookies_file();
    driver.get("https://www.reverso.net/favorites/en").await?;

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

