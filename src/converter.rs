//! Macro mappings:
//! - M1: + -
//! - M2: < >
//! - M3: ,
//! - M4: .
//! - M5: [
//! - M6: ]

use brainfuck::errors::*;
use brainfuck::{minimize, CellSize, Specifications, WriteString};
use std::io::Write;

#[allow(unused)]
pub fn convert<W>(src: &str, output: &mut W, specs: &Specifications) -> Result<()>
where
    W: Write,
{
    let minimized = minimize(src);
    let bytes = minimized.as_bytes();

    output.write_line(C_INIT)?;
    output.write_line(&compose_c_buf_init_str(&specs.cell_bits))?;
    output.write_line(C_MAIN_START)?;
    output.write_line(&compose_c_main_init_str(&specs.cell_bits))?;

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
            b'.' => Command::PutChar(specs.cell_bits.clone()).commit(output)?,
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

const C_INIT: &str = include_str!("./data/c_init.c");

fn compose_c_buf_init_str(cell_size: &CellSize) -> String {
    format!("{} buf[0xffff];", cell_size.c_type())
}

fn compose_c_main_init_str(cell_size: &CellSize) -> String {
    format!(
        r"{0} *ptr = buf;
int c;",
        cell_size.c_type()
    )
}

const C_MAIN_START: &str = "int main() {";

const C_MAIN_END: &str = "return 0;\n}";

enum Command {
    Increase(i32),
    MoveRight(i32),
    GetChar,
    PutChar(CellSize),
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
            Command::PutChar(s) => output.write_line(&format!("M4({})", get_c_print_macro(s))),
            Command::StartWhile => output.write_line("M5"),
            Command::EndWhile => output.write_line("M6"),
        }
    }
}

trait GetCType {
    fn c_type(&self) -> String;
}

impl GetCType for CellSize {
    fn c_type(&self) -> String {
        format!("uint{}_t", self.bits_size())
    }
}

fn get_c_print_macro(cell_size: &CellSize) -> String {
    format!("printU{}", cell_size.bits_size())
}
