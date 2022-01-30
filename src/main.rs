
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::ops::RangeBounds;
use std::path::Path;
use std::hash::Hash;
use itertools::{Itertools, izip};
use std::ops;
use rayon::prelude::*;
use text_io::read;

use std::{iter::Map, cmp::Reverse};


fn validate_set<C>(inc: &HashSet<C>, exc: &HashSet<C>) -> bool
where
    C: Hash + Eq
{
    inc.intersection(exc).count() == 0
}

fn validate<C>(inc: &[C], exc: &[C]) -> bool
where
    C: Hash + Eq
{
    validate_set(&HashSet::from_iter(inc), &HashSet::from_iter(exc))
}

fn subset_words(words: &[String], response: &Response) -> Vec<String>
{
    // inc: &[char], exc: &[char]
    let p : Vec<_> = words.iter().filter(|&word|
        {
            if response.exc.chars().any(|letter| word.contains(letter) ) {
                return false;
            }
            if !response.inc.chars().all(|letter| word.contains(letter)) {
                return false;
            }
            if !word.chars().zip(response.correct.chars()).all(|(w,c)|  c == ' ' || w == c) {
                return false;
            }
            response.neg_locate.iter().all(|neg| neg.chars().zip(word.chars()).all(|(n,w)| n == ' ' || n != w))
            // if neg_locate do not match
            // true
        }
        // if word includes value in exc then false
        // if word !include all value in inc then false
        // else true

    // true
    ).cloned().collect();
    // words.to_vec()
    p
}

/// Return a score for all words as used to split/solve the list of active words (worked out from Response)
fn score(words: &Vec<String>, response: &Response) -> Vec<(String, usize, usize,usize,)> {
    let active_words = subset_words(&words, response);
    println!("There are {} valid words left", active_words.len());

    // Use all words as viable guesses
    words.par_iter()
        .map(
            |guess_word| {
                let mut correct_tot = 0;
                let mut matches_tot = 0;
                let mut rejected_tot = 0;

                // For each target word score it based on
                for target_word in &active_words {
                    let potential_response = check_guess(&target_word, &guess_word) + response;
                    let reduced_words = subset_words(&active_words, &potential_response);

                    let (correct, matches) = potential_response.score();
                    // panic!("Testing {} and {} to score {}  and {}", target_word, )
                    correct_tot += correct;
                    matches_tot += matches;
                    rejected_tot += active_words.len()-reduced_words.len();
                };
            (guess_word.to_string(), correct_tot, matches_tot, rejected_tot)
            }
        )
        .collect()

  }
fn top_matches(scored: &Vec<(String, usize, usize, usize,)>, guesses: usize) -> Vec<(String, usize, usize,usize,)> {
    scored.clone().into_iter()
        .sorted_by_key(|v| Reverse(v.1))
        .take(guesses)
        .collect_vec()
}
fn top_correct(scored: &Vec<(String, usize, usize, usize,)>, guesses: usize) -> Vec<(String, usize, usize,usize,)> {
    scored.clone().into_iter()
        .sorted_by_key(|v| Reverse(v.2))
        .take(guesses)
        .collect_vec()
}
fn top_rejected(scored: &Vec<(String, usize, usize, usize,)>, guesses: usize) -> Vec<(String, usize, usize, usize,)> {
    scored.clone().into_iter()
        .sorted_by_key(|v| Reverse(v.3))
        .take(guesses)
        .collect_vec()
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct Guess{
    guess: String,
    reply: String,
}
impl Guess{
    fn new(guess: &str, reply: &str) -> Guess {
        Guess { guess: guess.to_string(), reply: reply.to_string()}
    }
    fn inc(&self) -> String {
        izip!(self.guess.chars(), self.reply.chars()).filter_map(|(g,r)| if r == 'g' || r=='y'  {Some(g)} else {None} ).unique().collect()
    }
    fn exc(&self) -> String {
        izip!(self.guess.chars(), self.reply.chars()).filter_map(|(g,r)| if r == 'b'  {Some(g)} else {None} ).unique().collect()
    }
    fn yg_check(&self, word: &str) -> bool {
        izip!(self.guess.chars(), self.reply.chars(), word.chars()).all(|(g,r,w)| {
            // Check greens match and yellows do not match
            match r {
                'g' => g==w,
                'y' => g!=w,
                _ => true,
            }
        } )
    }
    fn slow_check(&self, word: &str) -> bool {
        self.exc().chars().all(|e| !word.contains(e)) &&
        self.inc().chars().all(|i| word.contains(i)) &&
        self.yg_check(word)
        // self.reply.chars().zip(self.chars()).zip(word.chars())
    }
}
#[derive(PartialEq, Eq, Debug)]
struct SResponse{
    guesses: Vec<Guess>,
    inc: String,
    exc: String
}

impl SResponse {
    fn new(guesses: &Vec<Guess>) -> SResponse {
        let exc = guesses.iter().map(|guess| guess.exc()).fold("".to_string(), |acc, exc| acc+ &exc).chars().unique().collect();
        let inc = guesses.iter().map(|guess| guess.inc()).fold("".to_string(), |acc, exc| acc+ &exc).chars().unique().collect();
        SResponse { guesses: guesses.to_vec(), exc: exc, inc: inc}
    }
    fn slow_check(&self, word: &str) -> bool {
        self.guesses.iter().all(|guess| guess.slow_check(word))
    }
    fn check(&self, word: &str) -> bool {
        self.exc.chars().all(|e| !word.contains(e)) &&
        self.inc.chars().all(|i| word.contains(i)) &&
        self.guesses.iter().all(|guess| guess.yg_check(word))
    }
}


impl ops::Add<&SResponse> for SResponse {
    type Output = Self;

    fn add(self, rhs: &SResponse) -> SResponse {
        let exc = (self.exc + &rhs.exc).chars().unique().collect();
        let inc = (self.inc + &rhs.inc).chars().unique().collect();
        let mut result = SResponse{ guesses: self.guesses.clone() , exc: exc, inc: inc};

        result.guesses.append(&mut rhs.guesses.to_vec());

        result
    }
}

#[cfg(test)]
mod tests0 {

    use super::*;
    #[test]
    fn test_guess_compare() {
        assert_eq!(Guess::new("a", "b"), Guess::new("a", "b"));
    }
    #[test]
    fn test_response_compare() {
        assert_eq!(SResponse::new(&vec!()), SResponse::new(&vec!()));
    }
    #[test]
    fn test_guess_inc() {
        assert_eq!(Guess::new("", "").inc(), "");

        assert_eq!(Guess::new("a", "b").inc(), "");
        assert_eq!(Guess::new("a", "y").inc(), "a");
        assert_eq!(Guess::new("ab", "yb").inc(), "a");
        assert_eq!(Guess::new("ba", "by").inc(), "a");
        assert_eq!(Guess::new("ab", "yy").inc(), "ab");

        assert_eq!(Guess::new("a", "g").inc(), "a");
        assert_eq!(Guess::new("ab", "gb").inc(), "a");
        assert_eq!(Guess::new("ba", "bg").inc(), "a");
        assert_eq!(Guess::new("ab", "gg").inc(), "ab");

        assert_eq!(Guess::new("ab", "yg").inc(), "ab");
    }
    #[test]
    fn test_guess_exc() {
        assert_eq!(Guess::new("", "").exc(), "");

        assert_eq!(Guess::new("a", "b").exc(), "a");
        assert_eq!(Guess::new("a", "y").exc(), "");
        assert_eq!(Guess::new("ab", "yb").exc(), "b");
        assert_eq!(Guess::new("ba", "by").exc(), "b");
        assert_eq!(Guess::new("ab", "bb").exc(), "ab");

        assert_eq!(Guess::new("a", "g").exc(), "");
        assert_eq!(Guess::new("ab", "gb").exc(), "b");
        assert_eq!(Guess::new("ba", "bg").exc(), "b");
    }

    #[test]
    fn test_guess_slow_check() {
        // Check b
        assert_eq!(Guess::new("", "").slow_check(""), true);

        assert_eq!(Guess::new("a", "b").slow_check("a"), false);
        assert_eq!(Guess::new("a", "b").slow_check("b"), true);

        assert_eq!(Guess::new("ab", "bb").slow_check(""), true);
        assert_eq!(Guess::new("ab", "bb").slow_check("a"), false);
        assert_eq!(Guess::new("ab", "bb").slow_check("b"), false);
        assert_eq!(Guess::new("ab", "bb").slow_check("c"), true);


        // Check y
        assert_eq!(Guess::new("a", "y").slow_check("a"), false);
        assert_eq!(Guess::new("ab", "yb").slow_check("ca"), true);

        assert_eq!(Guess::new("ab", "yb").slow_check("ac"), false);

        // Check g
        assert_eq!(Guess::new("a", "g").slow_check("a"), true);

        assert_eq!(Guess::new("ab", "gb").slow_check("ac"), true);
        assert_eq!(Guess::new("ab", "gb").slow_check("cb"), false);

        assert_eq!(Guess::new("ba", "bg").slow_check("ac"), false);
    }

    #[test]
    fn test_sresponse_check() {

        // Check b
        assert_eq!(SResponse::new(&vec!(Guess::new("a", "b"))).slow_check(""), true);
        assert_eq!(SResponse::new(&vec!(Guess::new("a", "b"))).check(""), true);


        assert_eq!(SResponse::new(&vec!(Guess::new("ab", "bb"))).slow_check("cc"), true);
        assert_eq!(SResponse::new(&vec!(Guess::new("ab", "bb"))).check("cc"), true);
        assert_eq!(SResponse::new(&vec!(Guess::new("ab", "bb"))).slow_check("a"), false);
        assert_eq!(SResponse::new(&vec!(Guess::new("ab", "bb"))).check("a"), false);
        assert_eq!(SResponse::new(&vec!(Guess::new("ab", "bb"))).slow_check("b"), false);
        assert_eq!(SResponse::new(&vec!(Guess::new("ab", "bb"))).check("b"), false);

        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "bb"),
            Guess::new("cd", "bb"),
        )).slow_check("ee"), true);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "bb"),
            Guess::new("cd", "bb"),
        )).check("ee"), true);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "bb"),
            Guess::new("cd", "bb"),
        )).slow_check("ae"), false);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "bb"),
            Guess::new("cd", "bb"),
        )).check("ae"), false);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "bb"),
            Guess::new("cd", "bb"),
        )).slow_check("ea"), false);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "bb"),
            Guess::new("cd", "bb"),
        )).check("ea"), false);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "bb"),
            Guess::new("cd", "bb"),
        )).slow_check("ec"), false);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "bb"),
            Guess::new("cd", "bb"),
        )).check("ec"), false);

        // Check y
        assert_eq!(SResponse::new(&vec!(Guess::new("a", "y"))).slow_check("a"), false);
        assert_eq!(SResponse::new(&vec!(Guess::new("a", "y"))).check("a"), false);
        assert_eq!(SResponse::new(&vec!(Guess::new("ab", "yb"))).slow_check("ac"), false);
        assert_eq!(SResponse::new(&vec!(Guess::new("ab", "yb"))).check("ac"), false);
        assert_eq!(SResponse::new(&vec!(Guess::new("ab", "yb"))).slow_check("ca"), true);
        assert_eq!(SResponse::new(&vec!(Guess::new("ab", "yb"))).check("ca"), true);

        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "yb"),
            Guess::new("cd", "by"),
        )).slow_check("da"), true);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "yb"),
            Guess::new("cd", "by"),
        )).check("da"), true);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "yb"),
            Guess::new("cd", "by"),
        )).slow_check("df"), false);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "yb"),
            Guess::new("cd", "by"),
        )).check("df"), false);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "yb"),
            Guess::new("cd", "by"),
        )).slow_check("fa"), false);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "yb"),
            Guess::new("cd", "by"),
        )).check("fa"), false);

        // Check g
        assert_eq!(SResponse::new(&vec!(Guess::new("a", "g"))).slow_check("a"), true);
        assert_eq!(SResponse::new(&vec!(Guess::new("a", "g"))).check("a"), true);
        assert_eq!(SResponse::new(&vec!(Guess::new("a", "g"))).slow_check("b"), false);
        assert_eq!(SResponse::new(&vec!(Guess::new("a", "g"))).check("b"), false);

        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "gb"),
            Guess::new("cd", "bb"),
        )).slow_check("ae"), true);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "gb"),
            Guess::new("cd", "bb"),
        )).check("ae"), true);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "gb"),
            Guess::new("cd", "bb"),
        )).slow_check("ee"), false);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "gb"),
            Guess::new("cd", "bb"),
        )).check("ee"), false);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "gb"),
            Guess::new("cd", "bg"),
        )).slow_check("ad"), true);
        assert_eq!(SResponse::new(&vec!(
            Guess::new("ab", "gb"),
            Guess::new("cd", "bg"),
        )).check("ad"), true);

    }

}




struct Score{
    word: String,
    correct_tot: usize,
    present_tot: usize
}


#[derive(PartialEq, Eq, Debug)]
struct Response{
    correct: String,
    inc: String,
    exc: String,
    neg_locate: Vec<String>
}
impl Response {
    fn new(correct: &str, inc: &str, exc: &str, neg_locate: Vec<String>) -> Response {
        Response { correct: correct.to_string(), inc: inc.to_string(), exc: exc.to_string(), neg_locate: neg_locate}
    }

    // Return score of correct matches and of number of characters matched
    fn score(self) -> (usize, usize) {
        (self.correct.chars().filter(|l| *l != ' ').count(), self.inc.len())
    }
}

/// check the guess and return a response object to describe the match/reject
fn check_guess(target: &str, guess: &str) -> Response {
    if target.len() != guess.len() {
        panic!("Target and guess but be same length");
    }
    let inc: String = guess.chars().filter(|g| target.contains(*g)).unique().sorted().collect();
    let correct: String = target.chars().zip(guess.chars()).map(|(t,g)| if t == g {t} else {' '}  ).collect();
    let mut neg_exists = false;
    let neg_locate: String = guess.chars().zip(correct.chars()).map(|(g,c)| if inc.contains(g) && g != c {neg_exists=true;g} else {' '}).collect();
    Response{
        correct: correct.clone(),
        inc: inc.clone(),
        exc: guess.chars().filter(|g| !target.contains(*g)).unique().sorted().collect(),
        neg_locate: if neg_exists {vec!(neg_locate)} else {Vec::new()}
    }
}

impl ops::Add<&Response> for Response {
    type Output = Self;

    fn add(self, rhs: &Response) -> Response {
        if self.correct.len() != rhs.correct.len() {
            panic!("lhs and rhs but be same length");
        }

        Response{
            correct: self.correct.chars().zip(rhs.correct.chars()).map(|(l, r)| if l == ' ' {r} else {l}).collect(),
            inc: (self.inc + &rhs.inc).chars().unique().sorted().collect(),
            exc: (self.exc + &rhs.exc).chars().unique().sorted().collect(),
            neg_locate: {
                let mut neg_locate = self.neg_locate.clone();
                neg_locate.extend(rhs.neg_locate.clone());
                neg_locate
            }
        }
    }
}
static WORD_LENGTH: usize = 5;

fn input_response() -> Result<Response, &'static str> {
    let mut line = String::new();
    println!("What is your guess");
    let readsize = std::io::stdin().read_line(&mut line).unwrap();
    if readsize != WORD_LENGTH+1 {
        return Err("Invalid size of guess");
    }
    line.pop();
    let guess = line.clone();

    line.clear();
    println!("What are colours of match (g)reen (y)ellow or (b)lack");
    let readsize = std::io::stdin().read_line(&mut line).unwrap();
    if readsize != WORD_LENGTH+1 {
        return Err("Invalid size of matches");
    }
    line.pop();
    let matches = line.clone();

    for c in matches.chars() {
        if ! "bgy".contains(c) {
            return Err("incorrect match char. Must be one of gby");
        }
    }
    let exc: String = guess.chars().zip(matches.chars()).filter_map(|(g, m)| if m == 'b' {Some(g)} else {None} ).collect();
    let inc: String = guess.chars().zip(matches.chars()).filter_map(|(g, m)| if "yg".contains(m) {Some(g)} else {None} ).collect();
    let correct: String = guess.chars().zip(matches.chars()).map(|(g,m)| if m == 'g' {g} else {' '}).collect();
    let neg_locate: String = guess.chars().zip(matches.chars()).map(|(g,m)| if m == 'y' {g} else {' '}).collect();

    Ok(Response::new(&correct, &inc, &exc, vec!(neg_locate)))
}


#[cfg(test)]
mod tests {
    use std::{iter::Map, cmp::Reverse};

    use super::*;
    #[test]
    fn test_check_guess() {
        assert_eq!(check_guess("abc", "def"), Response::new("   ", "", "def", vec!()));
        assert_eq!(check_guess("abc", "aef"), Response::new("a  ", "a", "ef", vec!()));
        assert_eq!(check_guess("abc", "daf"), Response::new("   ", "a", "df", vec!(" a ".to_string())));
        assert_eq!(check_guess("abc", "dea"), Response::new("   ", "a", "de", vec!("  a".to_string())));
        assert_eq!(check_guess("abc", "abc"), Response::new("abc", "abc", "", vec!()));
        assert_eq!(check_guess("aba", "abc"), Response::new("ab ", "ab", "c", vec!()));
        assert_eq!(check_guess("abc", "aba"), Response::new("ab ", "ab", "", vec!("  a".to_string())));
    }

    #[test]
    fn test_response_add() {
        assert_eq!(Response::new("","", "", Vec::new()) + &Response::new("", "", "", Vec::new()), Response::new("","", "", Vec::new()));
        assert_eq!(Response::new(" ","", "", Vec::new()) + &Response::new(" ", "", "", Vec::new()), Response::new(" ","", "", Vec::new()));
        assert_eq!(Response::new("a ","", "", Vec::new()) + &Response::new("  ", "", "", Vec::new()), Response::new("a ","", "", Vec::new()));
        assert_eq!(Response::new("  ","", "", Vec::new()) + &Response::new("a ", "", "", Vec::new()), Response::new("a ","", "", Vec::new()));
        assert_eq!(Response::new("a ","", "", Vec::new()) + &Response::new(" b", "", "", Vec::new()), Response::new("ab","", "", Vec::new()));
        assert_eq!(Response::new("","a", "", Vec::new()) + &Response::new("", "", "", Vec::new()), Response::new("","a", "", Vec::new()));
        assert_eq!(Response::new("","a", "", Vec::new()) + &Response::new("", "b", "", Vec::new()), Response::new("","ab", "", Vec::new()));
        assert_eq!(Response::new("","ba", "", Vec::new()) + &Response::new("", "", "", Vec::new()), Response::new("","ab", "", Vec::new()));
        assert_eq!(Response::new("","ab", "", Vec::new()) + &Response::new("", "a", "", Vec::new()), Response::new("","ab", "", Vec::new()));
    }

    #[test]
    fn test_score() {
        assert_eq!(Response::new("", "", "", Vec::new()).score(), (0,0));
        assert_eq!(Response::new("a", "", "", Vec::new()).score(), (1,0));
        assert_eq!(Response::new(" a", "", "", Vec::new()).score(), (1,0));
        assert_eq!(Response::new("ab", "", "", Vec::new()).score(), (2,0));
        assert_eq!(Response::new("", "a", "", Vec::new()).score(), (0,1));
        assert_eq!(Response::new("", "ab", "", Vec::new()).score(), (0,2));
        assert_eq!(Response::new(" b ", "abc", "", Vec::new()).score(), (1,3));
    }

    #[test]
    fn test_file_load() {
        let words = lines_from_file("sgb-words.txt").expect("Loaded words");

        assert_eq!(words.len(), 5757);
    }

    #[test]
    fn cross_sections() {
        let words = lines_from_file("sgb-words.txt").expect("Loaded words");

        assert_eq!(words.len(), 5757);

        // for each guess_word try every target word and give a score on how well it matches
        let response = Response::new("     ", "", "", Vec::new());
        let scored = score(&words, &response);
        let guess_words = top_correct(&scored, 100);

        println!("Best score words {:?}", guess_words);
        panic!("OK HERE");
    }


}




fn main() {
    println!("Hello, world!");
    let filename = "sgb-words.txt";

    static ASCII_LOWER: [char; 26] = [
        'a', 'b', 'c', 'd', 'e',
        'f', 'g', 'h', 'i', 'j',
        'k', 'l', 'm', 'n', 'o',
        'p', 'q', 'r', 's', 't',
        'u', 'v', 'w', 'x', 'y',
        'z',
    ];


    let target_word = "sugar";

    let guess: Vec<_> = "apple".chars().collect();






    // let letters_inc: Vec<_> = "".chars().collect();
    // let letters_exc: Vec<_> = "".chars().collect();
    // let matches: Vec<_> = "".chars().collect();

    // // if validate(&HashSet::from_iter(letters_inc), &HashSet::from_iter(letters_exc)) {
    // if validate(&letters_inc, &letters_exc) {
    //     println!("Was valid");
    // } else {
    //     println!("Was invalid");
    // }


    if let Ok(words) = lines_from_file(filename) {
        println!("There are {} lines in file", words.len());
        let mut response = Response::new("     ", "", "", vec!());

        while(true) {
            // let response = Response::new("     ", "", "", Vec::new());
            match input_response() {
                Ok(ir) => response = response+&ir,
                Err(err_msg) => println!("Failed to process response with error: {}", err_msg),
            }
            println!("Response is {:?}", response);



            let scored = score(&words, &response);
            let guess_correct = top_correct(&scored, 20);
            let guess_matches = top_matches(&scored, 20);

            println!("Correct: {:?}", guess_correct);
            println!("Matches: {:?}", guess_matches);
            println!("Rejected: {:?}", top_rejected(&scored, 20));

        }
    }

    //     // Work out how many words include each letter
    //     let residual : HashMap<_, _>= ASCII_LOWER.iter().map(|f| {
    //         (
    //             f,
    //             lines.iter()
    //                 .filter(|word| word.contains(*f))
    //                 .count()
    //             // format!("letter-{}", f)
    //         )
    //     }).collect();

    //     println!("Words remaining after elimination = {:?}", residual);

    //     // Work out which words eliminate the most other words

    //     // let lines: Vec<_> = vec!("mol", "vol", "tol", "bra", "cha").iter().map(|&s| s.into()).collect();



    //     let subset = subset_words(&lines, &letters_inc, &letters_exc);
    //     println!("There are {} words", subset.len());

    //     let options: Vec<_> = subset.iter().filter(|&word|
    //         word.chars().enumerate().any(|(index, letter)| letter == matches[index] )
    //     ).collect();
    //     println!("There are {} options: {:?}", options.len(), options);


    //     let mut residual : Vec<_> = subset.iter()
    //         .map(|word| {
    //             // For each word in list return word and count of words matching
    //             (
    //                 word,
    //                 options.iter().filter(|wordsearch| {
    //                     word.chars().any(|letter| wordsearch.contains(letter))
    //                 }).count()
    //             )
    //         }).collect();
    //     residual.sort_by_key(|value1| value1.1);
    //     println!("Words remaining after elimination = {:?}", residual);
    //}



    // Work out how many words are eliminated by each letter



    // if let Ok(lines) = read_lines(filename) {
    //     for line in lines {
    //         if let Ok(word) = line {
    //             println!("{}", word);
    //         }
    //     }
    // }
}

fn lines_from_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let buf = BufReader::new(file);
    Ok(buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
