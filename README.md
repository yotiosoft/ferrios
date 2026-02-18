# FerriOS

A toy OS in Rust🦀, based on blog\_os from [Writing an OS in Rust](https://os.phil-opp.com) and inspired by [xv6](https://github.com/mit-pdos/xv6-riscv).

## Background

[blog\_os](https://github.com/phil-opp/blog_os) is an excellent blog series that walks through OS development in Rust with remarkable clarity — it doesn't just show *what* to implement, but explains *why* each piece works the way it does. However, the series currently ends at the Async/Await chapter, leaving higher-level OS features uncharted.

FerriOS starts where blog_os leaves off. Taking blog_os as its foundation and [xv6](https://github.com/mit-pdos/xv6-riscv) as a reference for Unix-like OS design, the goal is to build a fully functional OS that reaches — and eventually exceeds — xv6 in capability, while staying true to Rust's strengths: memory safety, expressive type system, and fearless concurrency.

**Note:** FerriOS is a personal hobby project and is not intended as a learning resource or tutorial. While the code is publicly available, please keep in mind that it comes with no guarantees of correctness, completeness, or instructional value.

# 準備

nightly の設定、rust-src のインストール
```bash
$ rustup override set nightly
$ rustup component add rust-src
```

bootimage のインストール
```bash
$ rustup component add llvm-tools-preview
$ cargo install bootimage
```

# ビルド
```bash
$ cargo bootimage
```

# 起動
GUI で起動
```bash
$ qemu-system-x86_64 -drive format=raw,file=target/x86_64-ferrios/debug/bootimage-ferrios.bin
```

CUI で起動
```bash
$ qemu-system-x86_64 -nographic -serial mon:stdio -drive format=raw,file=target/x86_64-ferrios/debug/bootimage-ferrios.bin
```
