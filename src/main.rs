#![feature(portable_simd)]

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

    /// Test/demo mode: provide the target answer upfront, and it will automatically score the colors for your guesses.
    #[arg(short, long)]
    answer: Option<String>,
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

        if let Some(ref ans) = cli.answer {
            println!("Demo mode active! Target answer is set to: '{}'", ans);
        }

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

                let score_map: std::collections::HashMap<_, _> = scored.iter().map(|(w, s)| (w, s)).collect();

                println!("\n🌟 Top Recommended Guesses:");
                println!("{:-<75}", "");
                println!("{:<10} | {:<10} | {:<12} | {:<12} | {:<12}", "Word", "Entropy", "Eliminates", "Green Score", "Possible Ans?");
                println!("{:-<75}", "");

                for (word, entropy, is_possible) in entropies.iter().take(10) {
                    let sc = score_map.get(word).unwrap();
                    let possible_str = if *is_possible { "✓ Yes" } else { "✗ No" };
                    println!("{:<10} | {:<10.3} | {:<12} | {:<12} | {:<12}",
                             word, entropy, sc.rejected_words, sc.green, possible_str);
                }
                println!("{:-<75}\n", "");

                // Prompt user for their guess
                loop {
                    match prompt_guess() {
                        Ok(val) => break val,
                        Err(e) => println!("Error: {}", e),
                    }
                }
            };

            // Calculate colors if answer is provided, otherwise prompt user
            let ir = if let Some(ref ans) = cli.answer {
                let g = Guess::from_target(ans, &guess_str);
                println!("\n🤖 Auto-scoring guess '{}' against answer '{}'", guess_str, ans);
                let ir = SResponse::new(&vec![g]);
                ir
            } else {
                // Prompt user for the result of that guess
                let mut temp = None;
                loop {
                    match prompt_color(&guess_str) {
                        Ok(val) => {
                            temp = Some(val);
                            break;
                        }
                        Err(err_msg) => println!("Failed to process response with error: {}", err_msg),
                    }
                }
                temp.unwrap()
            };

            response = response + &ir;

            if let Some(ref ans) = cli.answer {
                if guess_str == *ans {
                    println!("\n🎉 You successfully solved it! The answer was '{}'.", ans);
                    break;
                }
            }
        }
    } else {
        println!("Failed to load word list: {}", filename);
    }
}
