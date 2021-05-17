use std::fs;
use std::io;
use std::io::{BufRead, Write};
use std::option::Option;
use std::path::PathBuf;

use regex::Regex;
use structopt::StructOpt;

//Globals

const REGEX_STR: &str = r#"([\w|_]+)\s+\(0[x|X]([0-9|a-z|A-Z]+),\s+"([\w|_]+)"\)"#;

// Opt

#[derive(Debug, StructOpt)]
#[structopt(name = "IDCDump")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    #[structopt(parse(from_os_str))]
    output: PathBuf,
}

// Helpers

fn parse_line(r: &Regex, line: &str) -> Option<(String, usize)> {
    let entry = r.captures(line)?;

    if &entry[1] == "set_name" {
        return Some((
            entry[3].to_string(),
            usize::from_str_radix(&entry[2], 16).unwrap(),
        ));
    }

    None
}

fn is_c_name(name: &str) -> bool {
    name.starts_with("@") || name.starts_with("_")
}

fn is_cxx_name(name: &str) -> bool {
    name.starts_with("_Z") ||   // Itanium names
    name.starts_with("?") ||    // Microsoft names
    name.starts_with("g_") ||   // Globals
    name.contains("::") // Unmangled nested cxx names
}

fn should_dump(name: &str) -> bool {
    is_cxx_name(name) || is_c_name(name)
}

// Entrypoint

fn main() -> io::Result<()> {
    let mut total_lines = 0;
    let mut dumped_funcs = 0;
    let r = Regex::new(REGEX_STR).unwrap();
    let opt = Opt::from_args();

    let input = fs::File::open(opt.input)?;
    let input = io::BufReader::new(input);
    let output = fs::File::create(opt.output)?;
    let mut file = io::LineWriter::new(output);

    for line in input.lines() {
        match parse_line(&r, &line?) {
            Some(v) => {
                total_lines += 1;
                if should_dump(&v.0) {
                    file.write_all(format!("0x{:x}: {}\n", v.1, v.0).as_bytes())?;
                    dumped_funcs += 1;
                }
            }
            None => (),
        }
    }

    println!(
        "Dumped {} names ({}% of the DB)",
        dumped_funcs,
        (dumped_funcs * 100) / total_lines
    );
    Ok(())
}
