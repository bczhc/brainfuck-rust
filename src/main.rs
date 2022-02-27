#![feature(mixed_integer_ops)]

mod lib;

use crate::lib::{check_brackets, minimize, CellSize, EofBehavior, Specifications};
use clap::{Arg, Command};
use std::fs::File;
use std::io::{stdin, stdout, Read};

mod converter;
mod interpreter;

mod errors;
use errors::{Error, Result};

fn main() -> Result<()> {
    let matches = Command::new("brainfuck")
        .bin_name("bf")
        .author("bczhc <bczhc0@126.com>")
        .about("Naive Brainfuck interpreter")
        .arg(
            Arg::new("src")
                .id("src")
                .required(false)
                .help("Source file. If not given, read source from stdin."),
        )
        .arg(
            Arg::new("eof-behavior")
                .id("eof-behavior")
                .takes_value(true)
                .short('E')
                .long("eof")
                .required(false)
                .default_value("zero")
                .help("EOF behavior")
                .possible_values(["zero", "neg1", "NC"])
                .ignore_case(true),
        )
        .arg(
            Arg::new("minimize")
                .id("minimize")
                .takes_value(false)
                .required(false)
                .short('m')
                .long("minimize")
                .help("Minimize the source code"),
        )
        .arg(
            Arg::new("convert")
                .id("convert")
                .takes_value(false)
                .required(false)
                .short('c')
                .long("convert")
                .help("Convert to C source code"),
        )
        .arg(
            Arg::new("cell-size")
                .id("cell-size")
                .takes_value(true)
                .required(false)
                .default_value("8")
                .possible_values(["8", "16", "32", "64"])
                .short('s')
                .long("cell-size")
                .help("Specify the size of the cells in bits"),
        )
        .get_matches();

    let mut stdout = stdout();
    let mut stdin = stdin();

    let mut src = String::new();
    if matches.is_present("src") {
        let src_path = matches.value_of("src").unwrap();
        let mut file = File::open(src_path)?;
        file.read_to_string(&mut src)?;
    } else {
        stdin.read_to_string(&mut src)?;
    }

    if matches.is_present("minimize") {
        println!("{}", minimize(&src));
        return Ok(());
    }

    let specs = Specifications {
        eof_behavior: matches
            .value_of("eof-behavior")
            .unwrap()
            .parse::<EofBehavior>()
            .unwrap(),
        cell_bits: matches
            .value_of("cell-size")
            .unwrap()
            .parse::<CellSize>()
            .unwrap(),
    };

    if !check_brackets(&src) {
        return Err(Error::UnpairedBrackets);
    }

    if matches.is_present("convert") {
        converter::convert(&src, &mut stdout, &specs)?;
    } else {
        interpreter::start(&src, &mut stdin, &mut stdout, &specs)?;
    }

    Ok(())
}
