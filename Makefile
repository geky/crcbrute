
RUSTFLAGS += -Ctarget-cpu=native

# this indirection is so commands with env are easily copied on the terminal
CARGO ?= RUSTFLAGS="$(RUSTFLAGS)" cargo +nightly

.PHONY: all build
all build:
	$(CARGO) build --release
	cp target/release/crcbrute ./crcbrute

.PHONY: run
run:
	./crcbrute 32

.PHONY: test
test:
	$(CARGO) test --release -- --nocapture --color=always

.PHONY: clean
clean:
	$(CARGO) clean
	rm -f ./crcbrute
