use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use syntect::easy::HighlightFile;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

struct TwatOptions {
    show_all: bool,           // -A: equivalent to -vET
    number_nonblank: bool,    // -b
    show_ends: bool,          // -E
    number: bool,             // -n
    squeeze_blank: bool,      // -s
    show_tabs: bool,          // -T
    show_nonprinting: bool,   // -v
    highlight: bool,          // --highlight
}

impl TwatOptions {
    fn from_args(args: &[String]) -> Self {
        Self {
            show_all: args.contains(&"-A".to_string()),
            number_nonblank: args.contains(&"-b".to_string()),
            show_ends: args.contains(&"-E".to_string()) || args.contains(&"-e".to_string()),
            number: args.contains(&"-n".to_string()),
            squeeze_blank: args.contains(&"-s".to_string()),
            show_tabs: args.contains(&"-T".to_string()) || args.contains(&"-A".to_string()),
            show_nonprinting: args.contains(&"-v".to_string()) || args.contains(&"-A".to_string()) || args.contains(&"-e".to_string()) || args.contains(&"-T".to_string()),
            highlight: args.contains(&"--highlight".to_string()),
        }
    }
}

fn cat_file(filename: &str, options: &TwatOptions) -> io::Result<()> {
    if options.highlight {
        highlight_file(filename)?;
    } else {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut line_number = 1;
        let mut previous_line_empty = false;

        for line in reader.lines() {
            let mut line = line?;
            let mut print_line = true;

            if options.squeeze_blank && line.is_empty() {
                if previous_line_empty {
                    continue;
                }
                previous_line_empty = true;
            } else {
                previous_line_empty = false;
            }

            if options.show_tabs {
                line = line.replace("\t", "^I");
            }

            if options.show_nonprinting {
                line = line.chars()
                    .map(|c| if c.is_ascii() && !c.is_control() { c } else { '^' }).collect();
            }

            if options.show_ends {
                line.push('$');
            }

            if options.number_nonblank && !line.is_empty() {
                print_line = false;
                print!("{:6}\t", line_number);
                line_number += 1;
            } else if options.number {
                print_line = false;
                print!("{:6}\t", line_number);
                line_number += 1;
            }

            if print_line {
                println!("{}", line);
            } else {
                println!("{}", line);
            }
        }
    }

    Ok(())
}

fn highlight_file(filename: &str) -> io::Result<()> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"]; // You can choose any theme you prefer

    let mut highlighter = HighlightFile::new(filename, &ps, theme)?;

    let mut line = String::new();

    while highlighter.reader.read_line(&mut line)? > 0 {
        let regions = highlighter.highlight_lines.highlight(&line, &ps);
        let escaped = as_24_bit_terminal_escaped(&regions[..], true);
        print!("{}", escaped);
        line.clear();
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [options] <file1> <file2> ...", args[0]);
        std::process::exit(1);
    }

    let options = TwatOptions::from_args(&args[1..]);

    for arg in &args[1..] {
        if !arg.starts_with('-') {
            if let Err(e) = cat_file(arg, &options) {
                eprintln!("Error opening file {}: {}", arg, e);
            }
        }
    }

    Ok(())
}
