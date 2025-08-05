use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use genanki_rs::{Deck, Field, Model, Note, Template};
use rand::seq::SliceRandom;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{
    config,
    utils::{self, get_path},
};

const WORD_LIST_FILE_NAME: &str = "wordlist.json";

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
pub struct Word {
    word: String,
    definition: String,
    example_sentence: String,
    part_of_speech: String,
}

pub fn scrape_node(node: &String) -> Vec<Word> {
    let fragment = Html::parse_fragment(node);
    let mut results: Vec<Word> = Vec::new();

    let item_selector = Selector::parse("app-favourite-item-new").unwrap();
    let word_selector = Selector::parse("a.app-link").unwrap();
    let def_selector = Selector::parse("div.favourite-item-new__translate").unwrap();
    let example_selector = Selector::parse("div.favourite-item-new__example-source").unwrap();
    let pos_selector = Selector::parse("span.favourite-item-new__source-pos").unwrap();

    for item in fragment.select(&item_selector) {
        let word = item
            .select(&word_selector)
            .next()
            .map(|n| n.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let definition = item
            .select(&def_selector)
            .next()
            .map(|n| n.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let example = item
            .select(&example_selector)
            .next()
            .map(|n| n.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let part_of_speech = item
            .select(&pos_selector)
            .next()
            .map(|n| n.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        results.push(Word {
            word,
            definition,
            example_sentence: example,
            part_of_speech,
        });
    }

    results
}

fn read_list() -> Option<Vec<Word>> {
    let path = utils::get_path(WORD_LIST_FILE_NAME);
    if !Path::new(&path).exists() {
        File::create(&path).unwrap();
        println!("Created {}", WORD_LIST_FILE_NAME);
        return None;
    }

    println!("Found {}", WORD_LIST_FILE_NAME);
    let mut file = File::open(&path).unwrap();
    let mut file_out = String::new();
    let bytes_read = file.read_to_string(&mut file_out).unwrap();
    if bytes_read == 0 {
        return None;
    }

    let parsed =
        serde_json::from_str::<Vec<Word>>(&file_out).expect("Failed to deserialize {path}");

    Some(parsed)
}

fn write_list(words: &Vec<Word>) {
    let words_json = serde_json::to_string_pretty(words).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(get_path(WORD_LIST_FILE_NAME))
        .unwrap();

    file.write(&words_json.as_bytes()).unwrap();
    println!("Wrote to {}", WORD_LIST_FILE_NAME);
}

pub fn update_list(words: &Vec<Word>, cfg: &config::Config) {
    let mut final_list: Vec<Word>;
    let option_file_words = read_list();
    if let Some(file_words) = option_file_words {
        let hash_file: HashSet<Word> = file_words.clone().into_iter().collect();
        let mut diff: Vec<Word> = words
            .iter()
            .filter(|item| !hash_file.contains(&item))
            .cloned()
            .collect();

        if diff.len() == 0 {
            println!("Your saved words don't contain any new words. Nothing changed ");
            return;
        }

        println!("Found {} new words. Adding...", diff.len());
        final_list = file_words.clone();
        final_list.append(&mut diff);
    } else {
        final_list = words.clone();
    }

    write_list(&final_list);
    println!("Updated the wordlist.json");

    final_list.shuffle(&mut rand::rng());
    create_anki_deck(&final_list, cfg);
    println!("Updated the {}", cfg.anki_file_name);
}

fn create_anki_deck(words: &Vec<Word>, cfg: &config::Config) {
    let mut deck = Deck::new(cfg.deck.id, &cfg.deck.name[..], &cfg.deck.description[..]);
    let custom_css = ".card {\n font-family: arial;\n font-size: 20px;\n text-align: center;\n color: black;\n}\n";

    for word in words {
        let model =
            Model::new(
                cfg.model.id,
                &cfg.model.name[..],
                vec![
                    Field::new("Word"),
                    Field::new("Definition"),
                    Field::new("Example"),
                ],
                vec![Template::new("Card").qfmt("{{Word}}").afmt(
                    r#"{{FrontSide}}<hr id="definition">{{Definition}}<br/><br/>{{Example}}"#,
                )],
            )
            .css(custom_css);

        let note = Note::new(
            model,
            vec![
                &format!("{} ({})", word.word, word.part_of_speech)[..],
                &word.definition[..],
                &word.example_sentence[..],
            ],
        )
        .unwrap();

        deck.add_note(note);
    }

    deck.write_to_file(&get_path(&cfg.anki_file_name)).unwrap();
}
