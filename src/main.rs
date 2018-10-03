use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::mem;

fn main() -> Result<(), io::Error> {
    // Load all english words
    let english_words = File::open("english.txt")?;
    let reader = BufReader::new(&english_words);
    let mut words: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .map(|word| word.to_uppercase())
        .collect();

    words.dedup();

    // Get cryptogram input
    println!("Enter your cryptogram:");

    let mut cryptogram = String::new();

    io::stdin().read_line(&mut cryptogram)?;

    // Get the unique set of encrypted words.
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

    // Compute all possible words that the encrypted words could map to.
    let matches = compute_matches(&unsolved, &words);

    // Sort the list of matches so that the encrypted words with the smallest number of matches are
    // first in the list.
    let mut matches = matches.iter().collect::<Vec<_>>();
    matches.sort_by(|&(_, a), &(_, b)| a.len().cmp(&b.len()));

    // Create a set of possible keys to be evaluated, starts with a single empty key.
    let mut possible_keys = vec![HashMap::new()];

    // Go through each encrypted word.
    for (encrypted_word, possible_matches) in &matches {
        let mut prev_keys = mem::replace(&mut possible_keys, vec![]);
        prev_keys.dedup();

        // Try generating new keys based on the previous possible partial keys.
        for prev_key in &prev_keys {
            // Check each possible match for the current word.
            for solution in *possible_matches {
                // Check if this possible match is valid for the current key.
                let partial = compute_partial(encrypted_word, prev_key);
                if !compare_to_partial(solution, &partial) {
                    // Key not valid for this word, move to the next possible match.
                    continue;
                }

                // Generate a new partial key based on the current previous key.
                // If a valid key cannot be created, go to the next possible match.
                let key = match compute_partial_key(encrypted_word, solution, prev_key) {
                    Some(key) => key,
                    None => continue,
                };

                let mut key_valid = true;

                // Check if this key would cause any of the words to not be able to map to a single
                // english word.
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
                    // Store the key to evaluate the next possible keys using the next word.
                    possible_keys.push(key);
                }
            }
        }
    }

    // `possible_keys` contains all the valid solution keys. Use the first one for the solution.
    let key = match possible_keys.first() {
        Some(key) => key,
        None => {
            println!("Could not find solution.");
            return Ok(());
        }
    };

    // Using the key, convert the encrypted input to the decrypted output.
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

/// Finds the set of possible words that each encrypted word could map to based on the letter
/// pattern in each word.
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

/// Generates the letter pattern from the given word.
///
/// Each letter in word is represented by a new letter starting at A.
/// For each letter that hasn't been seen yet a new character is used to represent that letter.
///
/// Example:
/// `"TEST"` -> `"ABCA"`
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

/// Given a previous partial key, an encrypted word, and a possible match, creates a new partial key.
/// If a mapping of lettwers from the encrypted word to the possible match contradicts the current
/// key, the key is invalid and `None` is returned.
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

/// Creates a partial solution to an encrypted word from the given partial key.
///
/// If the mapping exists for a character, the character is mapped, otherwise the character is
/// represented by a `.`.
///
/// Example:
/// encrypted word: `DVMC`
/// key: `[D: T, C: A]`
/// partial solution: `T..A`
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

/// Compares a partial solution to a possible match.
/// Returns true if the partial *could* map to the given possible match.
///
/// Example:
/// partial: `T..T`
/// possible match: `THAT`
/// Returns true
///
/// Example:
/// partial: `T..T`
/// possible match: `THIS`
/// Returns false
fn compare_to_partial(word: &str, partial: &str) -> bool {
    assert_eq!(word.len(), partial.len());

    for (a, b) in word.chars().zip(partial.chars()) {
        if b != '.' && a != b {
            return false;
        }
    }

    true
}
