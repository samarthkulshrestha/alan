use std::fs;
use std::result;
use std::fmt::Write;
use std::env;
use std::iter::Peekable;
use std::process::ExitCode;

type Result<T> = result::Result<T, ()>;

#[derive(Debug, PartialEq)]
struct Symbol<'a> {
    name: &'a str,
}

#[derive(Debug)]
enum Step {
    Left,
    Right,
}

#[derive(Debug)]
struct Case<'a> {
    state: Symbol<'a>,
    read: Symbol<'a>,
    write: Symbol<'a>,
    step: Step,
    next: Symbol<'a>,
}

#[derive(Debug)]
struct Machine<'a> {
    state: Symbol<'a>,
    tape: Vec<Symbol<'a>>,
    tape_default: Symbol<'a>,
    head: usize,
    halt: bool,
}

impl<'a> Machine<'a> {
    fn next(&mut self, cases: &[Case<'a>]) -> Result<()> {
        for case in cases {
            if case.state == self.state && case.read == self.tape[self.head] {
                self.tape[self.head].name = case.write.name;
                match case.step {
                    Step::Left => {
                        if self.head == 0 {
                            eprintln!("ERROR: tape underflow.");
                            return Err(());
                        }
                        self.head -= 1;
                    }
                    Step::Right => {
                        self.head += 1;
                    }
                }
                self.state.name = case.next.name;
                self.halt = false;
                break;
            }
        }
        Ok(())
    }

    fn print(&self) {
        let mut buffer = String::new();
        let mut head = 0;

        let _ = write!(&mut buffer, "{state}: ", state = self.state.name);
        for (i, symbol) in self.tape.iter().enumerate() {
            if i == self.head {
                head = buffer.len();
            }
            let _ = write!(&mut buffer, "{name} ", name = symbol.name);
        }
        println!("{buffer}");
        // TODO: use the field width formatting of println
        for _ in 0..head {
            print!(" ");
        }
        println!("^");
    }
}

fn parse_symbol<'a>(lexer: &mut impl Iterator<Item = &'a str>) -> Result<Symbol<'a>> {
    if let Some(name) = lexer.next() {
        Ok(Symbol{name})
    } else {
        eprintln!("ERROR: expected symbol but reached end of input");
        Err(())
    }
}

fn parse_step<'a>(lexer: &mut impl Iterator<Item = &'a str>) -> Result<Step> {
    let symbol = parse_symbol(lexer)?;
    match symbol.name {
        "->" => Ok(Step::Right),
        "<-" => Ok(Step::Left),
        name => {
            eprintln!("ERROR: expected '->' or '<-' but got {name}");
            Err(())
        }
    }
}

fn parse_case<'a>(lexer: &mut impl Iterator<Item = &'a str>) -> Result<Case<'a>> {
    let state = parse_symbol(lexer)?;
    let read = parse_symbol(lexer)?;
    let write = parse_symbol(lexer)?;
    let step = parse_step(lexer)?;
    let next = parse_symbol(lexer)?;
    Ok(Case{state, read, write, step, next})
}

fn parse_cases<'a>(lexer: &mut Peekable<impl Iterator<Item = &'a str>>) -> Result<Vec<Case<'a>>> {
    let mut cases = vec![];
    while lexer.peek().is_some() {
        cases.push(parse_case(lexer)?);
    }

    Ok(cases)
}

fn parse_tape<'a>(lexer: &mut Peekable<impl Iterator<Item = &'a str>>) -> Result<Vec<Symbol<'a>>> {
    let mut symbols = vec![];
    while lexer.peek().is_some() {
        symbols.push(parse_symbol(lexer)?);
    }

    Ok(symbols)
}

fn usage(program: &str) {
    eprintln!("usage: {program} <input.alan> <input.tape>");
}

fn start() -> Result<()> {
    let mut args = env::args();
    let program = args.next().expect("program name is always present.");

    let alan_path;
    if let Some(path) = args.next() {
        alan_path = path;
    } else {
        usage(&program);
        eprintln!("ERROR: no input.alan provided.");
        return Err(());
    }
    let alan_source = fs::read_to_string(alan_path.clone()).map_err(|err| {
        eprintln!("ERROR: could not read file {alan_path}: {err}");
    })?;
    let cases = parse_cases(&mut alan_source.split(&[' ', '\n']).filter(|t| t.len() > 0).peekable())?;

    let tape_path;
    if let Some(path) = args.next() {
        tape_path = path;
    } else {
        usage(&program);
        eprintln!("ERROR: no input.tape provided.");
        return Err(());
    }
    let tape_source = fs::read_to_string(tape_path.clone()).map_err(|err| {
        eprintln!("ERROR: could not read file {tape_path}: {err}");
    })?;
    let tape = parse_tape(&mut tape_source.split(&[' ', '\n']).filter(|t| t.len() > 0).peekable())?;

    let tape_default;
    if let Some(symbol) = tape.last() {
        tape_default = symbol;
    } else {
        eprintln!("ERROR: tape file may not be empty.");
        return Err(());
    }

    let mut machine = Machine {
        state: Symbol{name: "Inc"},
        tape,
        tape_default,
        head: 0,
        halt: false,
    };

    while !machine.halt {
        machine.print();
        machine.halt = true;
        machine.next(&cases)?;
    }

    Ok(())
}

fn main() -> ExitCode {
    match start() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}
