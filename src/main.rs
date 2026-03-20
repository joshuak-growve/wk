use clap::{Parser, Subcommand};

mod kana;
mod kanji;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    Kana,
    Kanji,
}

fn main() {
    let cli = Cli::parse();
    match cli.mode {
        Mode::Kana => run_kana_practice(),
        Mode::Kanji => run_kanji_practice(),
    }
}

fn run_kana_practice() {
    println!("Starting Kana practice!");
    kana::run();
}

fn run_kanji_practice() {
    println!("Starting Kanji practice!");
    kanji::run();
}