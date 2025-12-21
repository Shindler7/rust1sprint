//! Взаимодействие с аргументами командной строки.

use clap::Parser;
use std::env;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    input_file: PathBuf,

    #[clap(short, long)]
    output_file: PathBuf,

    #[clap(short, long)]
    input_format: String,

    #[clap(short, long)]
    output_format: String,
}

fn cli_parse() {
    let args = Args::parse();
}

/// Предоставляет с помощью стандартных методов директорию проекта.
pub fn current_dir() -> PathBuf {
    env::current_dir().expect("Не удаётся получить директорию проекта")
}
