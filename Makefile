


sgb-words.txt:
	curl -O https://www-cs-faculty.stanford.edu/~knuth/sgb-words.txt


doc: cargo-doc

cargo-doc:
	@cargo doc --no-deps


bench: cargo-bench

cargo-bench:
	@cargo bench -- compare
	# @cargo bench -- Simd
