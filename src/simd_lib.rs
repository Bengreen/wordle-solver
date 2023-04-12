/* Information Theory with bitwise character operations

   Use Shannon Theory to work out the best moves
   https://www.quantamagazine.org/how-math-can-improve-your-wordle-score-20220525/


   Information = log2(1/probability)


   So the information we require to know the answer is log2(count of possibilities)


   So how do we work out the bet guess to make.

   So for given list of remaining wordles:
   For each given wordle it could be check for all words available (not just those in remaining wordles) and identify how much
   information will be given by each guess.

*/

use std::{
    fmt::{self, Display},
    ops::{self, Deref, DerefMut},
    simd::{u8x8, Simd, SimdPartialEq, Mask, mask8x8},
};

#[derive(Debug, PartialEq)]
pub struct SimdLetter {
    pub data: Simd<u8, 8>,
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub struct SimdWordle {
    pub data: Simd<u8, 8>,
}

impl SimdWordle {
    fn answer_for_guess(&self, guess: SimdWordle) -> SimdAnswer {
        let green = self.data.simd_eq(guess.data);
        // println!("Green = {:?}", green);
        // let mut my_array: [u8; 8] = [NONE; 8];

        let yellow = u8x8::splat(YELLOW);
        let black = u8x8::splat(BLACK);

        let yellow_mask: [bool;8] = guess.data.as_array().iter().map(|&letter| {
            // println!("{}/{} = {:?}", self, letter, self.data.simd_eq(u8x8::splat(letter)));
            self.data.simd_eq(u8x8::splat(letter)).any()
        }).collect::<Vec<_>>().try_into().unwrap();

        let yel_mask = mask8x8::from(yellow_mask);

        // println!("yellow mask = {:?}", yellow_mask);

        let myreply = green.select(u8x8::splat(GREEN), yel_mask.select(yellow, black));


        SimdAnswer { data: myreply }
    }
}

impl From<&str> for SimdWordle {
    fn from(item: &str) -> Self {
        let temp_padded = format!("{:*<8}", item);
        SimdWordle {
            data: u8x8::from_slice(temp_padded.as_bytes()),
        }
    }
}
impl From<&SimdWordle> for String {
    fn from(value: &SimdWordle) -> Self {
        let byte_array = value.data.to_array();

        // let without_padding: Vec<_> = byte_array.into_iter()
        //     .filter(|letter| *letter != '_' as u8)
        //     .collect();

        let string = std::str::from_utf8(&byte_array).unwrap().to_owned();
        string
    }
}

impl fmt::Display for SimdWordle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let byte_array = self.data.to_array();
        // let string = std::str::from_utf8(&byte_array).unwrap().to_owned();
        let string: String = self.into();
        write!(f, "{}", string)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct WordleVec(Vec<SimdWordle>);

impl Display for WordleVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, value) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", value)?;
        }
        write!(f, "]")
    }
}

impl DerefMut for WordleVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for WordleVec {
    type Target = Vec<SimdWordle>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Into<String>> From<Vec<S>> for WordleVec {
    fn from(words: Vec<S>) -> Self {
        WordleVec(
            words
                .into_iter()
                .map(|word| {
                    // let temp_string : String= word.into();
                    word.into().as_str().into()
                })
                .collect(),
        )
    }
}

#[deprecated(since = "0.2.0", note = "please use `WordleVec::into()` instead")]
pub fn word_to_simdwordle<S: Into<String>>(words: Vec<S>) -> WordleVec {
    WordleVec(
        words
            .into_iter()
            .map(|word| {
                // let temp_string : String= word.into();
                word.into().as_str().into()
            })
            .collect(),
    )
}

const YELLOW: u8 = 'y' as u8;
const GREEN: u8 = 'g' as u8;
const BLACK: u8 = 'b' as u8;
const NONE: u8 = '_' as u8;
const ANY: u8 = '*' as u8;

#[derive(Debug, PartialEq, Clone)]
struct SimdAnswer {
    pub data: Simd<u8, 8>,
}

impl From<&SimdAnswer> for String {
    fn from(value: &SimdAnswer) -> Self {
        let byte_array = value.data.to_array();

        // let without_padding: Vec<_> = byte_array.into_iter()
        //     .filter(|letter| *letter != '_' as u8)
        //     .collect();

        let string = std::str::from_utf8(&byte_array).unwrap().to_owned();
        string
    }
}


impl fmt::Display for SimdAnswer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let byte_array = self.data.to_array();
        // let string = std::str::from_utf8(&byte_array).unwrap().to_owned();
        let string: String = self.into();
        write!(f, "{}", string)
    }
}


impl From<&str> for SimdAnswer {
    fn from(item: &str) -> Self {
        if item.len() < 1 || item.len() > 8 {
            panic!("Guess must be between 1 and 8 chars, was {}", item.len())
        };

        let item_lower = item.to_ascii_lowercase();
        if !item_lower.chars().all(|letter| "gyb".contains(letter)) {
            panic!("Guess must contain only gyb")
        };

        // let mx: String = item.to_ascii_lowercase().chars().map(|letter| letter).collect();

        let temp_padded = format!("{:_<8}", item_lower);

        SimdAnswer {
            data: u8x8::from_slice(temp_padded.as_bytes()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct WordleFilter {
    // Positive values use NONE as their blanks so they do not accidentially match
    positive_all: WordleVec,
    positive_any: WordleVec,
    // Negative values use ANY as their blanks so they always match
    negative: WordleVec,
}

impl Display for WordleFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "={}", self.positive_all)?;
        write!(f, ",")?;

        write!(f, "+{}", self.positive_any)?;
        write!(f, ",")?;

        write!(f, "-{}", self.negative)?;
        write!(f, "")
    }
}

impl WordleFilter {
    pub fn new() -> WordleFilter {
        WordleFilter {
            positive_all: WordleVec(vec![]),
            positive_any: WordleVec(vec![]),
            negative: WordleVec(vec![]),
        }
    }
}

impl ops::Mul<SimdWordle> for WordleFilter {
    type Output = WordleFilter;

    fn mul(self, rhs: SimdWordle) -> Self::Output {
        let mut my_filter = self.clone();
        my_filter.positive_all.push(rhs);
        my_filter
    }
}
impl ops::Add<SimdWordle> for WordleFilter {
    type Output = WordleFilter;

    fn add(self, rhs: SimdWordle) -> Self::Output {
        let mut my_filter = self.clone();
        my_filter.positive_any.push(rhs);
        my_filter
    }
}
impl ops::Sub<SimdWordle> for WordleFilter {
    type Output = WordleFilter;

    fn sub(self, rhs: SimdWordle) -> Self::Output {
        let mut my_filter = self.clone();
        my_filter.negative.push(rhs);
        my_filter
    }
}

impl ops::Add<SimdGuess> for WordleFilter {
    type Output = WordleFilter;

    fn add(self, rhs: SimdGuess) -> Self::Output {
        let mut my_filter = self.clone();

        my_filter.positive_all.push(rhs.green());
        my_filter.negative.append(&mut rhs.black());
        let (mut neg, mut pos) = rhs.yellow();

        my_filter.positive_any.append(&mut pos);
        my_filter.negative.append(&mut neg);

        my_filter
    }
}

#[derive(Debug, PartialEq)]
struct SimdGuess {
    guess: SimdWordle,
    reply: SimdAnswer,
}

impl SimdGuess {
    pub fn new<S: Into<String>, T: Into<String>>(guess: S, reply: T) -> SimdGuess {
        // let guess_string: String = guess.into();
        // let reply_string: String = reply.into();

        SimdGuess {
            guess: SimdWordle::from(guess.into().as_str()),
            reply: SimdAnswer::from(reply.into().as_str()),
        }
    }
    pub fn green(&self) -> SimdWordle {
        // mask out where replies are not green
        let green_simd = u8x8::splat(GREEN);
        let blank_simd = u8x8::splat(ANY);
        let my_mask = self.reply.data.simd_eq(green_simd);
        SimdWordle {
            data: my_mask.select(self.guess.data, blank_simd),
        }
    }
    pub fn black(&self) -> WordleVec {
        // Masks to match to all letters matching
        // let my_mask = self.guess.data.simd_eq(BLANK_SIMD);
        let none_simd = u8x8::splat(NONE);
        let black_simd = u8x8::splat(BLACK);

        let my_mask = self.reply.data.simd_eq(black_simd); // identify with mask the values that are BLACK
        let my_vals = my_mask.select(self.guess.data, none_simd); // transfer the values OR BLANK

        WordleVec(
            my_vals
                .as_array()
                .iter()
                .filter_map(|&letter| {
                    // For each BLANK return None to filter else return splatted SimWordle
                    if letter == NONE {
                        None
                    } else {
                        Some(SimdWordle {
                            data: u8x8::splat(letter),
                        })
                    }
                })
                .collect(),
        )
    }

    pub fn yellow(&self) -> (WordleVec, WordleVec) {
        // Mask out guess where replies are not yellow
        // This does not capture that the resultant words must include the letter somewhere
        let yellow_simd = u8x8::splat(YELLOW);
        let any_simd = u8x8::splat(ANY);

        let yellow_mask = self.reply.data.simd_eq(yellow_simd); // identify with mask the values that are YELLOW
        let yellow_vals = yellow_mask.select(self.guess.data, any_simd); // transfer the values OR BLANK

        (
            // Negative reply to reject matches on yellow digit with the letter
            WordleVec(
                yellow_vals
                    .as_array()
                    .iter()
                    .enumerate()
                    .filter_map(|(index, &letter)| {
                        // For each BLANK return None to filter else return splatted SimWordle
                        if letter == ANY {
                            None
                        } else {
                            let mut my_array: [u8; 8] = [NONE; 8];
                            my_array[index] = self.guess.data.as_array()[index];

                            Some(SimdWordle {
                                data: Simd::from(my_array),
                            })
                        }
                    })
                    .collect(),
            ),
            // Positive to match on at least one of
            WordleVec(
                yellow_vals
                    .as_array()
                    .iter()
                    .enumerate()
                    .filter_map(|(index, &letter)| {
                        // For each BLANK return None to filter else return splatted SimWordle
                        if letter == ANY {
                            None
                        } else {
                            let mut my_array: [u8; 8] = [letter; 8];
                            my_array[index] = ANY;

                            Some(SimdWordle {
                                data: Simd::from(my_array),
                            })
                        }
                    })
                    .collect(),
            ),
        )
    }
}

pub trait BitCodedFunctions {
    fn simd_filter(&self, filter: &WordleFilter) -> WordleVec;
    fn information_required(&self) -> f64;
}

impl BitCodedFunctions for WordleVec {
    fn simd_filter(&self, filter: &WordleFilter) -> WordleVec {
        let any_simd = u8x8::splat(ANY);

        // Match up the positive_all matches
        let positive_all: Vec<_> = self
            .iter()
            .filter_map(|&word| {
                if filter.positive_all.iter().all(|filter_value| {
                    //Mask out values that are set to * and set as * on target words then compare for eq
                    let any_mask = filter_value.data.simd_eq(any_simd);
                    let match_vals = any_mask.select(any_simd, word.data);
                    // println!("DETAIL = {:?} - {:?}, {:?}", filter_value.data, any_mask, match_vals);
                    let compare = filter_value.data.simd_eq(match_vals).all();
                    // println!(
                    //     "Checking {} and {} as {} because {:?}",
                    //     filter_value,
                    //     word,
                    //     compare,
                    //     filter_value.data.simd_eq(word.data)
                    // );
                    compare
                }) {
                    Some(word)
                } else {
                    None
                }
            })
            .collect();
        let positive_all = WordleVec(positive_all);
        // println!("positive_all output = {}", positive_all);

        let positive: Vec<_> = positive_all
            .iter()
            .filter_map(|&word| {
                if filter.positive_any.iter().all(|filtered| {
                    let compare = filtered.data.simd_eq(word.data).any();
                    // println!(
                    //     "Checking {} and {} as {} because {:?}",
                    //     filtered,
                    //     word,
                    //     compare,
                    //     filtered.data.simd_eq(word.data)
                    // );
                    compare
                }) {
                    Some(word)
                } else {
                    None
                }
            })
            .collect();
        let positive = WordleVec(positive);
        // println!("positive output = {}", positive);

        let negative: Vec<_> = positive
            .iter()
            .filter_map(|&word| {
                if filter.negative.iter().all(|filtered| {
                    let compare = !filtered.data.simd_eq(word.data).any();
                    // println!(
                    //     "Checking {} and {} as {} because {:?}",
                    //     filtered,
                    //     word,
                    //     compare,
                    //     filtered.data.simd_eq(word.data)
                    // );
                    compare
                }) {
                    Some(word)
                } else {
                    None
                }
            })
            .collect();
        let negative = WordleVec(negative);
        // println!("negative output = {}", negative);

        negative
    }

    fn information_required(&self) -> f64 {
        f64::from(u32::try_from(self.len()).unwrap()).log2()
    }
}

#[cfg(test)]
mod simd_tests {
    use std::{
        mem,
        simd::{u8x8, Simd, SimdPartialEq}, collections::HashMap,
    };

    use crate::lines_from_file;
    use rand::distributions::{Distribution, Uniform};

    use super::*;
    #[test]
    fn test_extract_guess() {

        let target_word = SimdWordle::from("wahoo");
        let guess_word = SimdWordle::from("goner");

        // awoke, snout, wound
        let answer = target_word.answer_for_guess(guess_word);

        println!("{}/{} = {}", target_word, guess_word, answer);
    }

    fn max_b<K, V>(a_hash_map: &HashMap<K, V>) -> Option<&K>
    where
        V: PartialOrd,
    {
        a_hash_map
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(k, _v)| k)
    }

    #[test]
    fn test_informationmeasure() {
        const NUM_WORDS: usize=400;

        let words = lines_from_file("sgb-words.txt").unwrap();

        // let words_wv: WordleVec = words.into();

        println!("There are {} words", words.len());

        let mut rng = rand::thread_rng();
        let die = Uniform::from(0..words.len());

        let mut subwords = vec![];
        while subwords.len() < NUM_WORDS {
            let choice = die.sample(&mut rng);

            let new_word = words[choice].clone();
            if !subwords.contains(&new_word) {
                subwords.push(new_word);
            }
        }

        println!("Have now got {} words: {:?}", subwords.len(), subwords);
        let wv: WordleVec = subwords.into();
        let initial_info_bits = wv.information_required();
        println!("Have now got {} words: {} need {} information bits", wv.len(), wv, initial_info_bits);


        let self_answer = wv[0].answer_for_guess(wv[0]);

        println!("self answer for <{} -> {}> = {}", wv[0], wv[0], self_answer);

        let my_answer = wv[0].answer_for_guess(wv[1]);

        println!("     answer for <{} -> {}> = {}", wv[0], wv[1], my_answer);


        let mut info_count: HashMap<SimdWordle, f64> = HashMap::new();

        for target_word in wv.iter() {
            info_count.insert(*target_word, 0.0);
        }

        for target_word in wv.iter() {
            // println!("Searching for word: {}", target_word);
            for guess_word in wv.iter() {
                // println!("Attempting guess: {}", guess_word);
                let my_filter= WordleFilter::new();

                let answer = target_word.answer_for_guess(*guess_word);

                let my_answer = SimdGuess{guess: *guess_word, reply: answer.clone()};
                let my_filter = my_filter+my_answer;
                // println!("Filter = {}", my_filter);

                let filtered_words = wv.simd_filter(&my_filter);

                let new_info_bits = filtered_words.information_required();
                *info_count.get_mut(guess_word).unwrap() += new_info_bits;
                // println!("Target: {} guessing {} => {} [{}] .. {}", target_word, guess_word, answer, filtered_words.len(), initial_info_bits-new_info_bits);
            }
        }

        println!("Sum of guesses = {:?}", info_count);
        let best_guess = max_b(&info_count).unwrap();

        println!("Best guess was {}", *best_guess);


    }

    #[test]
    fn test_wordlevec() {
        let my_words = vec!["maple", "apple", "bread", "mouth"];

        let my_wv: WordleVec = my_words.into();

        println!("My wordlevec = {}", my_wv);
    }
    #[test]
    fn test_filter() {
        let my_words = WordleVec(vec![
            SimdWordle::from("maple"),
            SimdWordle::from("apple"),
            SimdWordle::from("bread"),
            SimdWordle::from("mouth"),
        ]);

        let my_filter = WordleFilter::new();
        println!("Filter = {}", my_filter);
        let filtered_words = my_words.simd_filter(&my_filter);
        assert_eq!(
            filtered_words,
            WordleVec(vec!(
                SimdWordle::from("maple"),
                SimdWordle::from("apple"),
                SimdWordle::from("bread"),
                SimdWordle::from("mouth"),
            )),
        );

        let my_filter = my_filter - SimdWordle::from("bbbbbbbb");
        println!("Subtracted Filter = {}", my_filter);
        let filtered_words = my_words.simd_filter(&my_filter);
        assert_eq!(
            filtered_words,
            WordleVec(vec!(
                SimdWordle::from("maple"),
                SimdWordle::from("apple"),
                SimdWordle::from("mouth"),
            )),
        );

        let my_filter = my_filter + SimdWordle::from("aaaaaaaa");
        println!("Added Filter = {}", my_filter);
        let filtered_words = my_words.simd_filter(&my_filter);
        assert_eq!(
            filtered_words,
            WordleVec(vec!(SimdWordle::from("maple"), SimdWordle::from("apple"),)),
        );

        let my_filter = my_filter * SimdWordle::from("a*******");
        println!("Mul Filter = {}", my_filter);
        let filtered_words = my_words.simd_filter(&my_filter);
        assert_eq!(filtered_words, WordleVec(vec!(SimdWordle::from("apple"),)),);
    }

    #[test]
    fn test_simdwordle() {
        println!("SimdWordle sizeof = {}", mem::size_of::<SimdWordle>());

        let myword = String::from("abcde");

        let mywordle = SimdWordle::from(myword.as_str());
        println!("mywordle = {:?}, = {}", mywordle, mywordle);

        let mywords = vec!["abcd", "def"];

        let mywordles = word_to_simdwordle(mywords);

        println!("My wordles = {:?}", mywordles);
    }

    #[test]
    fn test_answer() {
        let my_answer = String::from("ggybb");

        let answer = SimdAnswer::from(my_answer.as_str());

        println!("SimdAnswer = {:?}", answer);
    }

    #[test]
    fn test_guess() {
        let my_word = String::from("abcde");
        let my_answer = String::from("ggybb");

        let guess = SimdGuess::new(my_word, my_answer);

        println!("SimdGuess = {:?}", guess);

        println!("{:?}.green = {}", guess, guess.green());
        println!("{:?}.black = {}", guess, guess.black());
        let (yel_neg, yel_pos) = guess.yellow();
        println!("{:?}.yellow = ({}, {})", guess, yel_neg, yel_pos);

        let my_filter = WordleFilter::new();
        println!("Filter = {}", my_filter);

        let my_filter = my_filter + guess;
        println!("Filter after guess = {} = {:?}", my_filter, my_filter);
    }
    #[test]
    fn test_simd() {
        let a0: [i32; 4] = [-2, 0, 2, 4];
        let a1 = [10, 9, 8, 7];
        let zm_add = a0.zip(a1).map(|(lhs, rhs)| lhs + rhs);
        let zm_mul = a0.zip(a1).map(|(lhs, rhs)| lhs * rhs);

        // `Simd<T, N>` implements `From<[T; N]>
        let (v0, v1) = (Simd::from(a0), Simd::from(a1));
        // Which means arrays implement `Into<Simd<T, N>>`.
        assert_eq!(v0 + v1, zm_add.into());
        assert_eq!(v0 * v1, zm_mul.into());

        println!("v0={:?}, v1={:?}", v0, v1);

        let a2: [u8; 8] = [0, 2, 3, 4, 5, 0, 0, 0];
        let va = Simd::from(a2);

        let compare = u8x8::splat(4);

        let mush = va.simd_eq(compare);

        println!("mush = {:?}", mush);
        println!("mush all = {}", mush.any());
    }
}
