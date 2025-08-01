reverso to anki an app written in rust(a revolutionary, memory safe language that replaces all languages and is very niche and underground, and I should probably add the word rusty to the name of this repo just cus im badass) that uses [Selenium](https://www.selenium.dev/) to scrape saved words from reverso.net and put them into an anki deck

## Installation
Go to builds and download the appropriate archive for your os and platform.

If there's no build for your platform, install cargo, and in the root directory run
```sh
cargo build --release
```

the executable will be in ./target/release/reverso-to-anki(.exe)

## Usage
I may have went into too much detail in this explanation, so you can just go to [TLDR](#tldr) if you just want to get things done. Feel free to read this section if you encounter any problems before making an issue / a PR.

The app will create auxiliary files so you should put it in a separate folder.

<a id="bloat_consumers" href="https://ibb.co/V0pLyYQY" target="_blank">
If you're on windows</a>, I advise you to run the program from cmd/powershell because as of now, all the output including errors are printed to the console. To do that, open cmd/powerhsell, run cd directory/with/the/executable and run .\reverso-to-anki.exe

<a id="fields"></a>The app will initially create a `cfg.toml` and stop so you can configure it as you like. Some of these fields may be important. Here is a breakdown:

- `language` – language abbreviation that goes to the URL, as in https://www.reverso.net/favorites/en. Languages other than English haven't been tested, but there shouldn't be any trouble.
- `anki_file_name` – self-explanatory; changing it doesn't affect the deck's identification.
- `port` – port which the WebDriver will be using. If you don't understand the previous sentence, you don't need to worry about it.
- Fields under `model` – identifier for Anki; changing it will create a new deck without progress.
- Fields under `deck` – `name` and `id` are identifiers visible in Anki; you may want to change them.

<br/>

After configuring cfg.toml and start the program again, the program will look for `cookies.json` which contains login cookies for your account. If you haven't imported them from another instance of this app, or simply it's your first time launching the program, it will ask you to login into your account on reverso.net.

Some of the services you may be using for your account are unavailable in a webdriver, which is intended behavior of it and cannot be changed by me. Fortunately though, for google account you can click "Forgot password" to reset the password for your account, even if you used google to login. After you do that, you will be able to login with the email you used. 

After you login, the `cookies.json` file will be created and you will not need to repeat this process possibly in months. The app will grab your saved words, definitions and example sentences for them. It will save this data in `wordlist.json` **DO NOT** rename or delete this file if you want to keep the old words in the deck with the new ones. 

Using this `wordlist.json` it will create an anki deck that with the file name you specified (or the default one) in `anki_file_name`. Launch it with anki, it will create a deck / sync the new words with the existing deck.

## TLDR
Launch the app, first launch will create a config file and then exit. Go read [fields](#fields) for this. 

After that, start the program again. [Windows users](#bloat_consumers) are recommended to start it with cmd / powerhsell because all of the program's output is printed to stdout and stderr.

It will open a browser where you'll need to login to reverso.net; it will remember your account next time you will retrieve the words from the site.

A bunch of files will be created. You will only need the `.apkg` file, which will have the name you specified in `anki_file_name`. Open this file and it will add the deck / sync it with the previous version.

