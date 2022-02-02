

use criterion::{
    BenchmarkId,
    criterion_group,
    criterion_main,
    Criterion, black_box,
};

use wordle::{lines_from_file, score, SResponse, Guess};

fn bench_score(c: &mut Criterion) {

    let filename = "sgb-words.txt";

    let words = lines_from_file(filename).expect("load dict words");

    c.bench_function("File read", |b| b.iter(|| lines_from_file(black_box(filename)).expect("Good read")));

    let response0 = SResponse::new(&vec!(
        // Guess::new("arose", "gbbbg"),
    ));
    let response1 = SResponse::new(&vec!(
        Guess::new("arose", "gbbbg"),
    ));
    let response2 = SResponse::new(&vec!(
        Guess::new("arose", "gbbbg"),
        Guess::new("arose", "gbbbg"),
    ));

    let mut group = c.benchmark_group("Word Processing");
    for i in [10, 20, 30,40, 50, 60, 100, 150, 200] {
        let wordlist = words[0..i].to_vec();
        // Call to score function
        group.bench_with_input(BenchmarkId::new("Response-0", format_args!("{:?}", wordlist.len())), &wordlist,
            |b, i| b.iter(|| score(i, &response0)));
        group.bench_with_input(BenchmarkId::new("Response-1", format_args!("{:?}", wordlist.len())), &wordlist,
        |b, i| b.iter(|| score(i, &response1)));
        group.bench_with_input(BenchmarkId::new("Response-2", format_args!("{:?}", wordlist.len())), &wordlist,
        |b, i| b.iter(|| score(i, &response2)));
    }

    group.finish();

}


criterion_group!(benches, bench_score);
criterion_main!(benches);
