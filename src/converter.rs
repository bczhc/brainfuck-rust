use brainfuck::errors::*;
use brainfuck::{minimize, Specifications, WriteString};
use std::io::Write;

#[allow(unused)]
pub fn convert<W>(src: &str, output: &mut W, specs: &Specifications) -> Result<()>
where
    W: Write,
{
    let minimized = minimize(src);
    let bytes = minimized.as_bytes();

    output.write_line(C_STATIC_INIT)?;
    output.write_line(C_MAIN_START)?;
    output.write_line(C_MAIN_INIT)?;

    let mut i = 0_usize;
    while i < bytes.len() {
        match bytes[i] {
            b'+' | b'-' => {
                let mut count = 0_i32;
                let mut j = i;
                while j < bytes.len() {
                    count += match bytes[j] {
                        b'+' => 1,
                        b'-' => -1,
                        _ => {
                            break;
                        }
                    };
                    j += 1;
                }
                Command::Increase(count).commit(output)?;
                i = j;
                continue;
            }
            b'<' | b'>' => {
                let mut count = 0_i32;
                let mut j = i;
                while j < bytes.len() {
                    count += match bytes[j] {
                        b'>' => 1,
                        b'<' => -1,
                        _ => {
                            break;
                        }
                    };
                    j += 1;
                }
                Command::MoveRight(count).commit(output)?;
                i = j;
                continue;
            }
            b',' => Command::GetChar.commit(output)?,
            b'.' => Command::PutChar.commit(output)?,
            b'[' => Command::StartWhile.commit(output)?,
            b']' => Command::EndWhile.commit(output)?,
            _ => {
                unreachable!()
            }
        }
        i += 1;
    }

    output.new_line()?;
    output.write_line(C_MAIN_END)?;
    Ok(())
}

/// Macro mappings:
/// - M1: + -
/// - M2: < >
/// - M3: ,
/// - M4: .
/// - M5: [
/// - M6: ]
const C_STATIC_INIT: &str = r"#include <stdio.h>
#define M1(x) *ptr += x;
#define M2(x) ptr += x;
#define M3 c = getchar(); if (c != EOF) {*ptr = c;} else {*ptr = 0;}
#define M4 putchar(*ptr); fflush(stdout);
#define M5 while (*ptr) {
#define M6 }
unsigned char buf[0xffff];
";

const C_MAIN_INIT: &str = r"unsigned char *ptr = buf;
unsigned char c;";

const C_MAIN_START: &str = "int main() {";

const C_MAIN_END: &str = "return 0;\n}";

enum Command {
    Increase(i32),
    MoveRight(i32),
    GetChar,
    PutChar,
    StartWhile,
    EndWhile,
}

impl Command {
    fn commit<W>(&self, output: &mut W) -> std::io::Result<()>
    where
        W: Write,
    {
        match self {
            Command::Increase(x) => output.write_line(&format!("M1({})", x)),
            Command::MoveRight(x) => output.write_line(&format!("M2({})", x)),
            Command::GetChar => output.write_line("M3"),
            Command::PutChar => output.write_line("M4"),
            Command::StartWhile => output.write_line("M5"),
            Command::EndWhile => output.write_line("M6"),
        }
    }
}
