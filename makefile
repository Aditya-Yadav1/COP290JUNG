all: build

build:
	@cargo build --release

test:
	@cargo test  

coverage:
	@cargo tarpaulin 

ext1:
	@cargo build --release --features gui
	@./target/release/spreadsheet

check: fmt clippy

fmt:
	@cargo fmt --all --check

clippy:
	@cargo clippy --all-features -- -D warnings

docs:
	@cargo doc --open --all-features &
	@pdflatex -interaction=batchmode report.tex

clean:
	@cargo clean
	@rm -f report.aux report.log report.out report.pdf
	
.PHONY: all build test coverage ext1 check fmt clippy clean