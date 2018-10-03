use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::mem;

fn main() -> Result<(), io::Error> {
    println!("Enter your cryptogram:");

    let mut cryptogram = String::new();

    io::stdin().read_line(&mut cryptogram)?;

    let unsolved = cryptogram
        .replace(".", "")
        .replace(",", "")
        .replace(":", "")
        .replace(";", "")
        .replace("\"", "")
        .replace("!", "")
        .replace("'", "")
        .split_whitespace()
        .map(|word| word.to_uppercase())
        .collect::<HashSet<String>>();

    let english_words = File::open("english.txt")?;
    let reader = BufReader::new(&english_words);
    let mut words: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .map(|word| word.to_uppercase())
        .collect();

    words.dedup();

    let matches = compute_matches(&unsolved, &words);

    let mut matches = matches.iter().collect::<Vec<_>>();
    matches.sort_by(|&(_, a), &(_, b)| a.len().cmp(&b.len()));
    let mut possible_keys = vec![HashMap::new()];

    for (encrypted_word, possible_matches) in &matches {
        let mut prev_keys = mem::replace(&mut possible_keys, vec![]);
        prev_keys.dedup();

        for prev_key in &prev_keys {
            for solution in *possible_matches {
                let partial = compute_partial(encrypted_word, prev_key);
                if !compare_to_partial(solution, &partial) {
                    continue;
                }

                let key = match compute_partial_key(encrypted_word, solution, prev_key) {
                    Some(key) => key,
                    None => continue,
                };

                let mut key_valid = true;

                for (word, word_matches) in &matches {
                    let partial = compute_partial(word, &key);
                    let has_matches = word_matches
                        .iter()
                        .any(|possible| compare_to_partial(possible, &partial));

                    if !has_matches {
                        key_valid = false;
                    }
                }

                if key_valid {
                    possible_keys.push(key);
                }
            }
        }
    }

    let key = match possible_keys.first() {
        Some(key) => key,
        None => {
            println!("Could not find solution.");
            return Ok(());
        }
    };

    let mut solution = String::new();

    for letter in cryptogram.to_uppercase().chars() {
        if letter < 'A' || letter > 'Z' {
            solution.push(letter);
            continue;
        }

        let mapping = key.get(&letter).unwrap();
        solution.push(*mapping);
    }

    println!("Answer:\n{}", solution);

    Ok(())
}

fn compute_matches(unsolved: &HashSet<String>, words: &[String]) -> HashMap<String, Vec<String>> {
    let mut matches = HashMap::new();

    for encrypted_word in unsolved {
        let pattern = compute_pattern(encrypted_word);

        let word_matches = words
            .iter()
            .filter(|word| word.len() == encrypted_word.len())
            .filter(|word| pattern == compute_pattern(word))
            .map(|word| word.to_uppercase())
            .collect::<Vec<String>>();

        matches.insert(encrypted_word.clone(), word_matches);
    }

    matches
}

fn compute_pattern(word: &str) -> String {
    let mut found = HashMap::new();
    let mut current = 'A';
    let mut result = String::from("");

    let capitalized = word.to_uppercase();

    for letter in capitalized.chars() {
        if letter < 'A' || letter > 'Z' {
            result.push(letter);
            continue;
        }

        match found.get(&letter) {
            Some(&value) => result.push(value),
            None => {
                found.insert(letter, current);
                result.push(current);
                current = ((current as u8) + 1) as char;
            }
        }
    }

    result
}

fn compute_partial_key(
    encrypted_word: &str,
    word: &str,
    prev_key: &HashMap<char, char>,
) -> Option<HashMap<char, char>> {
    assert_eq!(encrypted_word.len(), word.len());

    let mut key = prev_key.clone();

    for (a, b) in encrypted_word.chars().zip(word.chars()) {
        for (from, to) in &key {
            if b == *to && a != *from {
                return None;
            }
        }
        key.insert(a, b);
    }

    Some(key)
}

fn compute_partial(word: &str, key: &HashMap<char, char>) -> String {
    let mut partial = String::from("");

    for letter in word.chars() {
        match key.get(&letter) {
            Some(&mapped_letter) => partial.push(mapped_letter),
            None => partial.push('.'),
        }
    }

    partial
}

fn compare_to_partial(word: &str, partial: &str) -> bool {
    assert_eq!(word.len(), partial.len());

    for (a, b) in word.chars().zip(partial.chars()) {
        if b != '.' && a != b {
            return false;
        }
    }

    true
}
