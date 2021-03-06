use std::{fs::File, io::{BufReader, self, BufRead}, path::Path, ops, collections::HashSet};

use itertools::{izip, Itertools};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use std::hash::Hash;


pub fn lines_from_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let buf = BufReader::new(file);
    Ok(buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect())
}


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

fn subset_words(words: &[String], response: &SResponse) -> Vec<String>
{
    words.iter().filter(|&word|
        response.check(word)
    ).cloned().collect()
}
#[derive(Debug)]
pub struct Score{
    pub inc_letters: usize,
    pub exc_letters: usize,
    pub green: usize,
    pub rejected_words: usize,
}
impl ops::Add<&Score> for Score {
    type Output = Self;

    fn add(self, rhs: &Score) -> Score {
        Score{inc_letters: self.inc_letters+rhs.inc_letters, exc_letters: self.exc_letters+rhs.exc_letters, green: self.green+rhs.green, rejected_words: self.rejected_words+rhs.rejected_words}
    }
}


#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Guess{
    guess: String,
    reply: String,
}
impl Guess{
    pub fn new(guess: &str, reply: &str) -> Guess {
        Guess { guess: guess.to_string(), reply: reply.to_string()}
    }
    fn from_target(target: &str, guess: &str) -> Guess {
        let reply = target.chars().zip(guess.chars()).map(|(t, g)| {
            if t==g {
                'g'
            } else if target.contains(g) {
                'y'
            } else {
                'b'
            }
        }).collect();
        Guess{guess: guess.to_string(), reply: reply}
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
pub struct SResponse{
    guesses: Vec<Guess>,
    inc: String,
    exc: String
}

impl SResponse {
    pub fn new(guesses: &Vec<Guess>) -> SResponse {
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
    fn green(&self) -> String {
        if self.guesses.len() == 0 {
            return "".to_string()
        }
        if self.guesses[0].guess.len() == 0 {
            return "".to_string()
        }
        let init = " ".repeat(self.guesses[0].guess.len());
        self.guesses.iter().fold(init, |acc, guess| izip!(acc.chars(), guess.reply.chars(), guess.guess.chars()).map(|(a,r,g)| if r=='g' {g} else {a} ).collect())
    }
    fn green_count(&self) -> usize {
        self.green().chars().filter(|l| *l != ' ').count()
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


/// Return a score for all words as used to split/solve the list of active words (worked out from Response)
pub fn score(words: &Vec<String>, response: &SResponse) -> Vec<(String, Score,)> {
    let active_words = subset_words(&words, response);
    // println!("There are {} valid words left", active_words.len());

    // if active_words.len() < 20 {
    //     println!("Active words: {:?}", active_words);
    // }

    // Use all words as viable guesses
    words.par_iter()
        .map(
            |guess_word| {
                let mut total = Score{inc_letters: 0, exc_letters: 0,green: 0 ,rejected_words: 0};

                // For each target word score it based on
                for target_word in &active_words {
                    let potential_response = SResponse::new(&vec!(Guess::from_target(target_word, guess_word)))+response;
                    let reduced_words = subset_words(&active_words, &potential_response);

                    let score = Score{
                        inc_letters: potential_response.inc.len(),
                        exc_letters: potential_response.exc.len(),
                        green: potential_response.green_count(),
                        rejected_words: active_words.len()-reduced_words.len(),
                    };

                    total= total + &score;
                };
            (guess_word.to_string(), total)
            }
        )
        .collect()

  }
