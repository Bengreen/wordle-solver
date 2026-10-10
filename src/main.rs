#![feature(portable_simd)]

use itertools::Itertools;
use wordle::{lines_from_file, SResponse, score, calculate_entropies, Guess};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

/// Wordle solver and assistant
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the wordlist file
    #[arg(short, long, default_value = "sgb-words.txt")]
    wordlist: String,

    /// Initial guess to start the solver with
    #[arg(short, long)]
    guess: Option<String>,
}

static WORD_LENGTH: usize = 5;

fn prompt_guess() -> Result<String, &'static str> {
    let mut line = String::new();
    println!("What is your guess?");
    let readsize = std::io::stdin().read_line(&mut line).unwrap();
    if readsize != WORD_LENGTH + 1 {
        return Err("Invalid size of guess");
    }
    line.pop();
    Ok(line)
}

fn prompt_color(guess: &str) -> Result<SResponse, &'static str> {
    let mut line = String::new();
    println!("What are colours of match for '{}' (g)reen (y)ellow or (b)lack?", guess);
    let readsize = std::io::stdin().read_line(&mut line).unwrap();
    if readsize != WORD_LENGTH + 1 {
        return Err("Invalid size of matches");
    }
    line.pop();
    let reply = line;

    for c in reply.chars() {
        if !"bgy".contains(c) {
            return Err("incorrect match char. Must be one of gby");
        }
    }

    Ok(SResponse::new(&vec![Guess::new(guess, &reply)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_load() {
        let words = lines_from_file("sgb-words.txt").expect("Loaded words");
        assert_eq!(words.len(), 5757);
    }
}

fn main() {
    let cli = Cli::parse();

    println!("Reading files...");
    let filename = &cli.wordlist;

    if let Ok(words) = lines_from_file(filename) {
        println!("There are {} words in file", words.len());
        let mut response = SResponse::new(&vec![]);
        let mut initial_guess = cli.guess;

        loop {
            // Determine the next guess. Use initial guess if provided, otherwise calculate suggestions.
            let guess_str = if let Some(g) = initial_guess.take() {
                g
            } else {
                // Provide the best guesses based on current information.
                let pb = ProgressBar::new(words.len() as u64);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                        .unwrap()
                        .progress_chars("#>-"),
                );
                pb.set_message("Calculating best guesses and entropies...");
                pb.enable_steady_tick(std::time::Duration::from_millis(100));

                // Calculate entropies (from master)
                let mut entropies = calculate_entropies(&words, &response);
                entropies.sort_by(|a, b| {
                    b.1.partial_cmp(&a.1)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.2.cmp(&a.2))
                });

                // Calculate scores (from simd branch)
                let scored = score(&words, &response, Some(&pb));
                pb.finish_and_clear();

                println!("Entropy: {:?}", entropies.iter().take(10).collect::<Vec<_>>());
                println!("Top 10 by Green Matches: {:?}", scored.iter().sorted_by_key(|(_word, score)| score.green).rev().take(10).collect::<Vec<_>>());
                println!("Top 10 by Included Letters: {:?}", scored.iter().sorted_by_key(|(_word, score)| score.inc_letters).rev().take(10).collect::<Vec<_>>());
                println!("Top 10 by Excluded Letters: {:?}", scored.iter().sorted_by_key(|(_word, score)| score.exc_letters).rev().take(10).collect::<Vec<_>>());
                println!("Top 10 by Rejected Words: {:?}", scored.iter().sorted_by_key(|(_word, score)| score.rejected_words).rev().take(10).collect::<Vec<_>>());
                println!();

                // Prompt user for their guess
                loop {
                    match prompt_guess() {
                        Ok(val) => break val,
                        Err(e) => println!("Error: {}", e),
                    }
                }
            };

            // Prompt user for the result of that guess
            loop {
                match prompt_color(&guess_str) {
                    Ok(ir) => {
                        response = response + &ir;
                        println!("Current Response Filter is {:?}", response);
                        break;
                    }
                    Err(err_msg) => println!("Failed to process response with error: {}", err_msg),
                }
            }
        }
    } else {
        println!("Failed to load word list: {}", filename);
    }
}
