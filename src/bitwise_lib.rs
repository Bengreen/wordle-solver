
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


const BITS_PER_CHAR: usize = 5;
const LETTER_MASK: u32 = (1<<BITS_PER_CHAR)-1;
const MULVAL: u32 = 1 | 1<<5 | 1<<10 | 1<<15 | 1<<20;


#[derive(Debug, PartialEq)]
pub struct BitCodedLetter {
    pub data: u32,
}

impl From<char> for BitCodedLetter {
    fn from(item: char) -> Self {
        BitCodedLetter { data: (item as u32) - ('a' as u32)+1 }
    }
}

impl From<BitCodedLetter> for char {
    fn from(item: BitCodedLetter) -> Self {
        char::from_u32(item.data + ('a' as u32) -1).unwrap()
    }
}



#[derive(Debug, PartialEq, Clone)]
pub struct BitCodedWordle {
    pub data: u32,
}


impl BitCodedWordle {
    pub fn new<S: Into<String>>(word: S) -> BitCodedWordle {
        let guess_string: String = word.into();

        let mycoded = BitCodedWordle::from(guess_string.as_str());

        mycoded
    }
    fn letter(&self, index: usize) -> BitCodedLetter {
        let temp = self.data>>(index* BITS_PER_CHAR);
        BitCodedLetter { data: temp&LETTER_MASK }
    }
    fn len(&self) -> usize {
        BITS_PER_CHAR
    }

    fn matches(&self, guess: &BitCodedGuess) -> bool {
        // For each letter in guess filter words that match green
        // Get green letters in guess by ANDing with 0x00001 then multiplying by 0x11111
        // to get
        // for index in 0..BITS_PER_CHAR {
        //     match (guess.reply) {
        //         3 => return false,

        //     }
        // }
        // For each letter in word filter words that do not contain in other location
        // For each letter in word filter words that contain letter


        // Check if green matches in correct place
        // Check if yellow included but not in provided location
        // Check if black matches

        true
    }
}
impl From<&str> for BitCodedWordle {
    fn from(item: &str) -> Self {
        if item.len() != 5 {panic!("Guess must be 5 chars, was {}", item.len())};
        let mut bin_encoded: u32 = 0;
        for (index, char ) in item.to_ascii_lowercase().chars().enumerate() {
            bin_encoded |= BitCodedLetter::from(char).data<<(index*BITS_PER_CHAR);
        }
        BitCodedWordle { data: bin_encoded }
    }
}

impl From<BitCodedWordle> for String {
    fn from(item: BitCodedWordle) -> Self {
        let mut mystring = String::new();

        for index in 0..item.len() {
            mystring.push(item.letter(index).into());
        }

        mystring
    }
}

trait BitCodedFunctions {
    fn filter(&self, guess: &BitCodedGuess) -> Vec<BitCodedWordle>;
    fn information_required(&self) -> f64;
}

impl BitCodedFunctions for Vec<BitCodedWordle> {
    fn filter(&self, guess: &BitCodedGuess) -> Vec<BitCodedWordle> {

        //Green filters
        // if green is not zero then use green as filter
        let mut my_filter = vec!();

        let green = guess.green_bitcoded();
        if green.data == 0 {
            my_filter = self.clone();
        } else {
            for wordle in self {
                if (wordle.data ^ green.data) != 0 {
                    my_filter.push(wordle.clone());
                }
            }
        }

        //Yellow filters

        //Black filters

        let mut result = vec!();

        for word in self {
            if word.matches(&guess) {
                result.push((*word).clone());
            }
        }

        result
    }

    fn information_required(&self) -> f64 {
        f64::from(u32::try_from(self.len()).unwrap()).log2()
    }

}


#[derive(Debug, PartialEq)]
struct BitCodedAnswer {
    pub data: u32,
}

impl BitCodedAnswer {
    pub fn new<S: Into<String>>(reply: S) -> BitCodedAnswer {

        BitCodedAnswer::from(reply.into().as_str())
    }
}

impl From<&str> for BitCodedAnswer {
    fn from(item: &str) -> Self {
        if item.len() != 5 {panic!("Guess must be 5 chars, was {}", item.len())};
        let mut bin_encoded: u32 = 0;
        for (index, char ) in item.to_ascii_lowercase().chars().enumerate() {

            let encoded_char = match char {
                'g' => 3,
                'y' => 1,
                'b' => 0,
                _ => panic!("Character is not valid in guess {}", char),
            };
            bin_encoded |= encoded_char<<(index*BITS_PER_CHAR);
        }
        BitCodedAnswer { data: bin_encoded }
    }
}



#[derive(Debug, PartialEq)]
struct BitCodedGuess {
    guess: BitCodedWordle,
    reply: BitCodedAnswer,
}

impl BitCodedGuess {
    pub fn new<S: Into<String>, T: Into<String>>(guess: S, reply: T) -> BitCodedGuess {
        // let guess_string: String = guess.into();
        // let reply_string: String = reply.into();

        BitCodedGuess{
            guess: BitCodedWordle::new(guess),
            reply: BitCodedAnswer::new(reply)
        }
    }
    pub fn green_bitcoded(&self) -> BitCodedWordle {
        // Mask out guess where replies are not green
        let reply_mask_single = (self.reply.data>>1) & (1 | 1<<5 | 1<<10 | 1<<15 | 1<<20);
        let reply_mask = reply_mask_single * 0b11111u32; // 5 bits

        BitCodedWordle { data: reply_mask & self.guess.data }
    }
    pub fn black_bitcoded(&self) -> Vec<BitCodedWordle> {
        // Mask out guess where replies are not yellow
        let mut reply = vec!();

        for index in 0..(BITS_PER_CHAR as u32) {
            // if (1<<(index*(BITS_PER_CHAR as u32)) & self.reply.data)==0 {
            if (0b11u32 & (self.reply.data>>(index*(BITS_PER_CHAR as u32))))==0 {
                reply.push(BitCodedWordle{data: MULVAL * (0b11111u32 & (self.guess.data>>(index*(BITS_PER_CHAR as u32))))});
            }
        }
        reply
    }
    pub fn yellow_bitcoded(&self) -> Vec<BitCodedWordle> {
        // Mask out guess where replies are not yellow
        let mut reply = vec!();

        for index in 0..(BITS_PER_CHAR as u32) {
            if (0b11u32 & (self.reply.data>>(index*(BITS_PER_CHAR as u32))))==1 {
                reply.push(BitCodedWordle{data:
                    (MULVAL ^ (1<<(index*(BITS_PER_CHAR as u32)))) * (0b11111u32 & (self.guess.data>>(index*(BITS_PER_CHAR as u32)))
                )});
            }
        }
        reply

    }

}




pub fn words_to_bitcodedwordle<S: Into<String>>(words: Vec<S>) -> Vec<BitCodedWordle> {

    words
        .into_iter()
        .map(|word| BitCodedWordle::new(word))
        .collect()
}

#[derive(PartialEq, Debug)]
pub struct SResponse {
    // Capture the guesses we make
    guesses: Vec<BitCodedGuess>,
    inc: Vec<BitCodedLetter>,
    exc: Vec<BitCodedLetter>,
}

// impl SResponse {
//     pub fn new(guesses: &Vec<BitCodedWordle>) -> SResponse {

//     }
// }


#[cfg(test)]
mod bitwise_tests {
    use std::simd::{Simd, u8x8, SimdPartialEq};

    use crate::lines_from_file;

    use super::*;


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


        let a2: [u8;8] = [0,2,3,4,5,0,0,0];
        let va = Simd::from(a2);

        let compare = u8x8::splat(4);

        let mush = va.simd_eq(compare);

        println!("mush = {:?}", mush);
        println!("mush all = {}", mush.any());
    }

    #[test]
    fn test_conversion() {
        let a: BitCodedLetter = 'a'.into();
        assert_eq!(a.data,1, "a must be converted to 1");

        let a_char: char = a.into();
        assert_eq!(a_char, 'a', "a converted back from 1");

        let z: BitCodedLetter = 'z'.into();
        assert_eq!(z.data,26, "z must be converted to 27");

        let z_char: char = z.into();
        assert_eq!(z_char, 'z', "z converted back from 26");
    }

    #[test]
    fn test_bitcodedanswer() {
        let ben = BitCodedAnswer::from("ggggg");

        assert_eq!(ben.data,3247203);
    }

    #[test]
    fn test_bitcodedguess() {
        let guess = BitCodedGuess::new("abcde", "gbyyb");

        println!("green for {:?} = {:?}", guess, guess.green_bitcoded());
        println!("black for {:?} = {:?}", guess, guess.black_bitcoded());
        println!("yellow for {:?} = {:?}", guess, guess.yellow_bitcoded());
    }

    #[test]
    fn test_constructor() {
        let ben = BitCodedWordle::new("apple");

        println!("My guess is {:?}", ben);
    }
    #[test]
    fn test_from() {
        let myword = "apple";
        let bitcoded = BitCodedWordle::from(myword);

        println!("myword = {:?}", bitcoded);

        let andback = String::from(bitcoded);

        println!("reply was {}", andback);

        assert_eq!(myword, andback);
    }

    #[test]
    fn test_words_to() {
        let mylist = vec!["WORDL", "APPLE"];

        words_to_bitcodedwordle(mylist);
    }

    #[test]
    fn test_simd_big_list() {
        let words = lines_from_file("sgb-words.txt").unwrap();

        println!("Loaded {} words from file ", words.len());

        let words_8bytes: Vec<_>= words.iter().map(|word| format!("{:0>8}", word)).collect();

        let words_simd: Vec<_> = words_8bytes.iter().map(|word| {
            let word_bytes = word.as_bytes();
            u8x8::from_slice(word_bytes)
        }).collect();

        println!("num words = {}", words_simd.len());

        let letter = 'a';
        let letter_simd = u8x8::splat(letter as u8);


        let words_with_a: Vec<_> = words_simd.iter().filter(|&word| word.simd_eq(letter_simd).any())
        .collect();

        println!("There are {} words with the letter 'a'", words_with_a.len());
    }



    #[test]
    fn test_big_list() {
        let words = lines_from_file("sgb-words.txt").unwrap();

        println!("Loaded {} words from file ", words.len());

        let bitcoded_words = words_to_bitcodedwordle(words);

        let word = "apple";

        let bitcoded_word = BitCodedWordle::new(word);

        if bitcoded_words.contains(&bitcoded_word) {
            println!("Found: {}", word);
        } else {
            println!("NOT FOUND: {}", word);
        }


        println!("Wordle requires {} bits of information", bitcoded_words.information_required());

        let my_guess = BitCodedGuess::new("apple", "ggbbb");

        let matching_words = bitcoded_words.filter(&my_guess);
        println!("Guess result is of length {} so needs {} bits", matching_words.len(), matching_words.information_required());



    }


}
