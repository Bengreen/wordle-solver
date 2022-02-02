
use itertools::Itertools;
use wordle::{lines_from_file, SResponse, score, Guess};




static WORD_LENGTH: usize = 5;

fn input_response() -> Result<SResponse, &'static str> {
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
    let reply = line.clone();

    for c in reply.chars() {
        if ! "bgy".contains(c) {
            return Err("incorrect match char. Must be one of gby");
        }
    }

    Ok(SResponse::new(&vec!(Guess::new(&guess,&reply))))
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
    println!("Reading files!");
    let filename = "sgb-words.txt";

    if let Ok(words) = lines_from_file(filename) {
        println!("There are {} lines in file", words.len());
        let mut response = SResponse::new(&vec!());

        loop {

            match input_response() {
                Ok(ir) => response = response+&ir,
                Err(err_msg) => println!("Failed to process response with error: {}", err_msg),
            }
            println!("Response is {:?}", response);

            let scored = score(&words, &response);


            println!("Green: {:?}", scored.iter().sorted_by_key(|(_word, score)| score.green  ).rev().take(10).collect::<Vec<_>>());
            println!("Inc: {:?}", scored.iter().sorted_by_key(|(_word, score)| score.inc_letters  ).rev().take(10).collect::<Vec<_>>());
            println!("Exc: {:?}", scored.iter().sorted_by_key(|(_word, score)| score.exc_letters  ).rev().take(10).collect::<Vec<_>>());
            println!("Rej: {:?}", scored.iter().sorted_by_key(|(_word, score)| score.rejected_words  ).rev().take(10).collect::<Vec<_>>());

        }
    }

}
