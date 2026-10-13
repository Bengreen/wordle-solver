use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Feedback {
    pub exact: usize,
    pub color: usize,
}

pub fn generate_palette(colors: usize) -> Vec<char> {
    let all_colors = vec!['R', 'G', 'B', 'Y', 'O', 'P', 'C', 'M', 'W', 'K'];
    let mut palette = Vec::new();
    for i in 0..colors.min(10) {
        palette.push(all_colors[i]);
    }
    // If more than 10 colors, fallback to A-Z
    if colors > 10 {
        let alphabet: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
        for i in 10..colors.min(26) {
            palette.push(alphabet[i]);
        }
    }
    palette
}

pub fn generate_combinations(positions: usize, colors: usize) -> Vec<String> {
    let palette = generate_palette(colors);
    let mut combos = Vec::new();

    if positions == 0 {
        return combos;
    }

    // Initialize with single colors
    for &c in &palette {
        combos.push(c.to_string());
    }

    for _ in 1..positions {
        let mut new_combos = Vec::new();
        for combo in &combos {
            for &c in &palette {
                new_combos.push(format!("{}{}", combo, c));
            }
        }
        combos = new_combos;
    }

    combos
}

pub fn score_guess(target: &str, guess: &str) -> Feedback {
    let mut exact = 0;
    let mut color = 0;

    let target_chars: Vec<char> = target.chars().collect();
    let guess_chars: Vec<char> = guess.chars().collect();

    let mut target_used = vec![false; target_chars.len()];
    let mut guess_used = vec![false; guess_chars.len()];

    // First pass: exact matches
    for i in 0..guess_chars.len() {
        if i < target_chars.len() && guess_chars[i] == target_chars[i] {
            exact += 1;
            target_used[i] = true;
            guess_used[i] = true;
        }
    }

    // Second pass: color matches
    for i in 0..guess_chars.len() {
        if !guess_used[i] {
            for j in 0..target_chars.len() {
                if !target_used[j] && guess_chars[i] == target_chars[j] {
                    color += 1;
                    target_used[j] = true;
                    break;
                }
            }
        }
    }

    Feedback { exact, color }
}

pub fn calculate_entropies(
    all_combos: &[String],
    active_combos: &[String],
) -> Vec<(String, f64, f64)> {
    let num_active = active_combos.len() as f64;

    if active_combos.is_empty() {
        return all_combos.iter().map(|c| (c.clone(), 0.0, 0.0)).collect();
    }

    let mut entropies: Vec<(String, f64, f64)> = all_combos
        .par_iter()
        .map(|guess_word| {
            let mut pattern_counts: HashMap<Feedback, usize> = HashMap::new();

            for target_word in active_combos {
                let feedback = score_guess(target_word, guess_word);
                *pattern_counts.entry(feedback).or_insert(0) += 1;
            }

            let mut entropy = 0.0;
            let mut expected_eliminations = 0.0;
            for count in pattern_counts.values() {
                let p = (*count as f64) / num_active;
                entropy -= p * p.log2();
                // If this pattern occurs, we will have `count` possibilities left.
                // We will have eliminated (num_active - count) possibilities.
                expected_eliminations += p * (num_active - *count as f64);
            }

            (guess_word.clone(), entropy, expected_eliminations)
        })
        .collect();

    entropies.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
    });

    entropies
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_guess() {
        assert_eq!(score_guess("RGBY", "RGBY"), Feedback { exact: 4, color: 0 });
        assert_eq!(score_guess("RGBY", "YBGR"), Feedback { exact: 0, color: 4 });
        assert_eq!(score_guess("RRYY", "RGBY"), Feedback { exact: 2, color: 0 });
        assert_eq!(score_guess("RGBY", "RRYY"), Feedback { exact: 2, color: 0 });
        assert_eq!(score_guess("RRRR", "RBBB"), Feedback { exact: 1, color: 0 });
        assert_eq!(score_guess("RBBB", "RRRR"), Feedback { exact: 1, color: 0 });
        assert_eq!(score_guess("RGBY", "WXYZ"), Feedback { exact: 0, color: 1 });
        assert_eq!(score_guess("RRYY", "YYRR"), Feedback { exact: 0, color: 4 });
    }

    #[test]
    fn test_generate_palette() {
        assert_eq!(generate_palette(4), vec!['R', 'G', 'B', 'Y']);
        assert_eq!(
            generate_palette(10),
            vec!['R', 'G', 'B', 'Y', 'O', 'P', 'C', 'M', 'W', 'K']
        );
        assert_eq!(generate_palette(12).len(), 12);
        assert_eq!(generate_palette(12)[11], 'L');
    }

    #[test]
    fn test_generate_combinations() {
        let combos = generate_combinations(4, 6);
        assert_eq!(combos.len(), 6_usize.pow(4));
        assert!(combos.contains(&"RRRR".to_string()));
        assert!(combos.contains(&"RGBY".to_string()));
    }

    #[test]
    fn test_calculate_entropies() {
        let combos = generate_combinations(2, 2);
        let entropies = calculate_entropies(&combos, &combos);
        assert_eq!(entropies.len(), 4);
    }
}
