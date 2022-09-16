mod directory_walker;

use clap::{Parser, ValueEnum};
use console::style;
use csv::Writer;
use dashmap::DashMap;
use directory_walker::walker;
use humantime::format_duration;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufReader, Read};
use std::ops::Deref;
use std::path::PathBuf;
use std::time::Instant;

/// A simple program for extracting word frequencies from large quantities of text file.
/// Developed for the SpeedType game project. Supports English only.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Root dir containing text files
    #[clap(short, long, value_parser = clap::value_parser!(PathBuf), value_name = "DIR")]
    input: PathBuf,

    /// Allowed file extension for text file
    #[clap(short, long, value_parser, value_name = "EXT")]
    extension: FileExt,

    /// CSV output file
    #[clap(short, long, value_parser = clap::value_parser!(PathBuf), value_name = "FILE")]
    output: PathBuf,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum FileExt {
    Txt,
}

const LOGO: &str = r#"
   _____                     _ _______            _______ _____  
  / ____|                   | |__   __|          |__   __|  __ \ 
 | (___  _ __   ___  ___  __| |  | |_   _ _ __   ___| |  | |__) |
  \___ \| '_ \ / _ \/ _ \/ _` |  | | | | | '_ \ / _ \ |  |  ___/ 
  ____) | |_) |  __/  __/ (_| |  | | |_| | |_) |  __/ |  | |     
 |_____/| .__/ \___|\___|\__,_|  |_|\__, | .__/ \___|_|  |_|     
        | |                          __/ | |                     
        |_|                         |___/|_|                     
"#;

const SUBTEXT: &str = "high-speed text processor for the SpeedType game project";

fn main() {
    let cli = Cli::parse();

    //Abort if input path does not exist
    if !cli.input.deref().try_exists().unwrap_or_default() {
        println!("{}", style("Input path does not exist.").red().bold(),);
        std::process::abort();
    }

    let root_path = cli.input.to_str().unwrap();
    let csv_path = cli.output.to_str().unwrap();

    let extension;
    match cli.extension {
        FileExt::Txt => extension = "txt",
    }

    println!(
        "{}\n{}\n",
        style(LOGO).cyan().bold(),
        style(SUBTEXT).green().bold()
    );

    println!(
        "Getting '{}' files from {}...",
        style(extension).yellow().bold(),
        style(root_path).yellow().bold(),
    );

    let txt_files = walker::walk(root_path, extension);
    let txt_count = txt_files.len().try_into().unwrap();

    if txt_count == 0 {
        println!(
            "{}",
            style("Input path does not contain any files matching the specified extension!")
                .red()
                .bold(),
        );
        std::process::abort();
    }

    println!("Got {} files!", style(txt_count).yellow().bold());

    let words = DashMap::new();

    let pb = ProgressBar::new(txt_count);

    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.green}] ({pos}/{len}, ETA {eta})",
        )
        .unwrap(),
    );

    println!("{}", style("Start processing...").cyan());

    let start = Instant::now();

    for txt_file in txt_files {
        let file = File::open(txt_file.path()).unwrap_or_else(|_| {
            println!(
                "Failed to open file {:?}",
                style(txt_file.path()).red().bold(),
            );
            std::process::abort();
        });

        let mut reader = BufReader::new(file);

        let mut buf = Vec::new();

        reader.read_to_end(&mut buf).unwrap_or_else(|_| {
            println!(
                "Failed to read file {:?}",
                style(txt_file.path()).red().bold(),
            );
            std::process::abort();
        });

        //read_to_string fails if it encounters non-utf-8 bytes
        let content = String::from_utf8_lossy(&buf).to_string();

        let split_lines = content.split("\n");

        split_lines.par_bridge().for_each(|line| {
            let split_words = line.split(" ");

            for word in split_words {
                //remove various special symbols from beginning or end of a word
                let word = word.trim_matches(&[
                    '\'', '"', '-', '&', '.', ',', ';', ':', ')', '(', ']', '[', '}', '{',
                ] as &[_]);

                //remove single chars
                if word.len() < 2 {
                    continue;
                }

                //remove words still containing special characters
                if !word.chars().all(|c| char::is_ascii_alphabetic(&c)) {
                    continue;
                }

                //remove words which have an uppercase letter after the beginning
                if word[1..].chars().any(|c| char::is_ascii_uppercase(&c)) {
                    continue;
                }

                let word = word.to_ascii_lowercase();

                //remove words missing vowels
                if word
                    .chars()
                    .all(|c| !['a', 'e', 'i', 'o', 'u'].contains(&c))
                {
                    continue;
                }

                //remove roman numerals
                if word
                    .chars()
                    .all(|c| ['i', 'v', 'x', 'l', 'c', 'd', 'm'].contains(&c))
                {
                    continue;
                }

                //add to map or increment counter
                *words.entry(word.to_string()).or_insert(0) += 1
            }
        });

        pb.inc(1);
    }
    pb.finish_and_clear();

    let duration = format_duration(start.elapsed()).to_string();

    let word_count = words.iter().count();

    println!(
        "Got {} unique words in {} ",
        style(word_count).green().bold(),
        style(duration).yellow().bold(),
    );

    println!("Writing output CSV to {}", style(csv_path).yellow().bold());
    let mut wtr = Writer::from_path(csv_path).expect("Couldn't open file for writing");

    words.clone().iter().for_each(|w| {
        wtr.write_record(&[w.key(), &w.value().to_string()])
            .expect("Failed to write value");
    });

    println!("{}", style("ALL DONE!").green().bold());
}
