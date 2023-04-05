
#![feature(portable_simd)]
#![feature(array_zip)]

use std::simd::{u8x8, SimdPartialEq, Simd};

use criterion::{
    BenchmarkId,
    criterion_group,
    criterion_main,
    Criterion, black_box,
};
use wordle::{lines_from_file, score, SResponse, Guess};
use wordle::simd_lib::{word_to_simdwordle, WordleFilter, SimdWordle, BitCodedFunctions};

fn compare_str(word: &str, letter: char) -> bool {
    word.contains(letter)
}
fn compare_simd_str(word: &Simd<u8, 8>, letter: &Simd<u8, 8>) -> bool {

    word.simd_eq(*letter).any()
}

fn compare_simd(word: &str, letter: char) -> bool {
    true
}



fn bench_filter(c: &mut Criterion) {
    let filename = "sgb-words.txt";

    let words = lines_from_file(filename).expect("load dict words");

    let words_simd = word_to_simdwordle(words);

    let my_filter = WordleFilter::new();
    let my_filter = my_filter + SimdWordle::from("aaaaaaaa");

    let mut group = c.benchmark_group("Contains");

    // group.bench_function("string_contains", |b| b.iter(|| compare_str(black_box(&word), black_box(letter))));
    group.bench_function("simd_contains", |b| b.iter(|| words_simd.simd_filter(black_box(&my_filter))));

    group.finish();

}



fn bench_simd(c: &mut Criterion) {


    let word=String::from("apple");
    let letter='a';


    let word_8bytes = format!("{:0>8}", word);
    let word_bytes = word_8bytes.as_bytes();
    let word_simd = u8x8::from_slice(word_bytes);

    let letter_simd = u8x8::splat(letter as u8);


    let mut group = c.benchmark_group("Compare");

    group.bench_function("compare_str", |b| b.iter(|| compare_str(black_box(&word), black_box(letter))));
    group.bench_function("compare_simd_str", |b| b.iter(|| compare_simd_str(black_box(&word_simd), black_box(&letter_simd))));
    group.bench_function("compare_simd", |b| b.iter(|| compare_simd(black_box(&word), black_box(letter))));

    group.finish();


}


fn compare_vec_simd<'a>(words: &'a Vec<Simd<u8, 8>>, letter: &'a Simd<u8, 8>) -> Vec<&'a Simd<u8, 8>> {
    words.iter().filter(|&word| word.simd_eq(*letter).any()).collect()
}

fn compare_vec<'a>(words: &'a Vec<String>, letter: char) -> Vec<&'a String> {
    words.iter().filter(|&word| word.contains(letter)).collect()
}


fn bench_vec_simd(c: &mut Criterion) {

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

    let mut group = c.benchmark_group("VecContains");

    group.bench_function("compare_str", |b| b.iter(|| compare_vec(black_box(&words), black_box(letter))));
    group.bench_function("compare_vec_simd", |b| b.iter(|| compare_vec_simd(black_box(&words_simd), black_box(&letter_simd))));


    group.finish();

}

criterion_group!(benches, bench_simd, bench_vec_simd, bench_filter);
// criterion_group!(benches, bench_vec_simd);
criterion_main!(benches);
