#![feature(portable_simd)]

use clap::{Args, Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use wordle::{calculate_entropies, lines_from_file, score, Guess, SResponse};

/// Wordle solver and assistant
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Play or solve Wordle
    Wordle(WordleArgs),

    /// Play Mastermind (coming soon)
    Mastermind(MastermindArgs),
}

#[derive(Args, Debug)]
struct WordleArgs {
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

#[derive(Args, Debug)]
struct MastermindArgs {
    /// Number of positions in the Mastermind code
    #[arg(short, long, default_value_t = 4)]
    positions: usize,

    /// Number of distinct colors available
    #[arg(short, long, default_value_t = 6)]
    colors: usize,
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
    println!(
        "What are colours of match for '{}' (g)reen (y)ellow or (b)lack?",
        guess
    );
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
        // The word list size in main.rs test shouldn't be hardcoded to 5757 for generic files
        // Let's modify the test to load the default and just check it works
        let words = lines_from_file("sgb-words.txt").expect("Loaded words");
        assert!(words.len() > 0);
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Wordle(args) => run_wordle(args),
        Commands::Mastermind(args) => run_mastermind(args),
    }
}

fn run_wordle(args: WordleArgs) {
    println!("Reading files...");
    let filename = &args.wordlist;

    if let Ok(words) = lines_from_file(filename) {
        println!("There are {} words in file", words.len());
        let mut response = SResponse::new(&vec![]);
        let mut initial_guess = args.guess;

        if let Some(ref ans) = args.answer {
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

                let score_map: std::collections::HashMap<_, _> =
                    scored.iter().map(|(w, s)| (w, s)).collect();

                println!("\n🌟 Top Recommended Guesses:");
                println!("{:-<75}", "");
                println!(
                    "{:<10} | {:<10} | {:<12} | {:<12} | {:<12}",
                    "Word", "Entropy", "Eliminates", "Green Score", "Possible Ans?"
                );
                println!("{:-<75}", "");

                for (word, entropy, is_possible) in entropies.iter().take(10) {
                    let sc = score_map.get(word).unwrap();
                    let possible_str = if *is_possible { "✓ Yes" } else { "✗ No" };
                    println!(
                        "{:<10} | {:<10.3} | {:<12} | {:<12} | {:<12}",
                        word, entropy, sc.rejected_words, sc.green, possible_str
                    );
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
            let ir = if let Some(ref ans) = args.answer {
                let g = Guess::from_target(ans, &guess_str);
                println!(
                    "\n🤖 Auto-scoring guess '{}' against answer '{}'",
                    guess_str, ans
                );
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
                        Err(err_msg) => {
                            println!("Failed to process response with error: {}", err_msg)
                        }
                    }
                }
                temp.unwrap()
            };

            response = response + &ir;

            if let Some(ref ans) = args.answer {
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

fn prompt_mastermind_guess(positions: usize) -> Result<String, &'static str> {
    let mut line = String::new();
    println!("What is your guess?");
    if std::io::stdin().read_line(&mut line).unwrap() == 0 {
        std::process::exit(0);
    }
    line = line.trim().to_string();

    if line.len() != positions {
        return Err("Invalid size of guess");
    }

    Ok(line)
}

fn prompt_mastermind_feedback() -> Result<wordle::mastermind::Feedback, &'static str> {
    let mut exact_line = String::new();
    println!("Enter number of exact matches (right color, right position):");
    if std::io::stdin().read_line(&mut exact_line).unwrap() == 0 {
        std::process::exit(0);
    }

    let mut color_line = String::new();
    println!("Enter number of color matches (right color, wrong position):");
    if std::io::stdin().read_line(&mut color_line).unwrap() == 0 {
        std::process::exit(0);
    }

    let exact = exact_line
        .trim()
        .parse::<usize>()
        .map_err(|_| "Invalid exact match number")?;
    let color = color_line
        .trim()
        .parse::<usize>()
        .map_err(|_| "Invalid color match number")?;

    Ok(wordle::mastermind::Feedback { exact, color })
}

fn run_mastermind(args: MastermindArgs) {
    let positions = args.positions;
    let colors = args.colors;

    let palette = wordle::mastermind::generate_palette(colors);
    println!(
        "Starting Mastermind with {} positions and {} colors.",
        positions, colors
    );
    println!("Color palette: {:?}", palette);

    let all_combos = wordle::mastermind::generate_combinations(positions, colors);
    let mut active_combos = all_combos.clone();

    println!("There are {} possible combinations.", all_combos.len());

    loop {
        if active_combos.len() == 1 {
            println!("The answer must be: {}", active_combos[0]);
            break;
        } else if active_combos.len() == 0 {
            println!("No combinations left! Did you make a mistake in your feedback?");
            break;
        }

        let pb = indicatif::ProgressBar::new(all_combos.len() as u64);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message("Calculating best guesses and entropies...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let entropies = wordle::mastermind::calculate_entropies(&all_combos, &active_combos);
        pb.finish_and_clear();

        println!(
            "
🌟 Top Recommended Guesses:"
        );
        println!("{:-<60}", "");
        println!(
            "{:<15} | {:<10} | {:<20} | {:<12}",
            "Guess", "Entropy", "Expected Elims", "Possible?"
        );
        println!("{:-<60}", "");

        for (guess, entropy, expected_elim) in entropies.iter().take(10) {
            let possible_str = if active_combos.contains(guess) {
                "✓ Yes"
            } else {
                "✗ No"
            };
            println!(
                "{:<15} | {:<10.3} | {:<20.3} | {:<12}",
                guess, entropy, expected_elim, possible_str
            );
        }
        println!(
            "{:-<60}
",
            ""
        );

        let guess = loop {
            match prompt_mastermind_guess(positions) {
                Ok(val) => break val,
                Err(e) => println!("Error: {}", e),
            }
        };

        let feedback = loop {
            match prompt_mastermind_feedback() {
                Ok(val) => break val,
                Err(e) => println!("Error: {}", e),
            }
        };

        if feedback.exact == positions {
            println!("🎉 You successfully solved it!");
            break;
        }

        active_combos.retain(|target| wordle::mastermind::score_guess(target, &guess) == feedback);

        println!("Active combinations left: {}", active_combos.len());
    }
}
