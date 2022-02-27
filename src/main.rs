#![feature(mixed_integer_ops)]

mod lib;

use crate::lib::{check_brackets, minimize, EofBehavior};
use clap::{Arg, Command};
use std::fs::File;
use std::io::{stdin, stdout, Read};
use std::str::FromStr;

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

    let eof_behavior = EofBehavior::from_str(matches.value_of("eof-behavior").unwrap()).unwrap();

    if !check_brackets(&src) {
        return Err(Error::UnpairedBrackets);
    }

    if matches.is_present("convert") {
        converter::convert(&src, &mut stdout)?;
    } else {
        interpreter::start(&src, &mut stdin, &mut stdout, &eof_behavior)?;
    }

    Ok(())
}
