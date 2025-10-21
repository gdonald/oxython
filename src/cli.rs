use crate::bytecode::OpCode;
use crate::compiler::Compiler;
use crate::object::ObjectType;
use crate::vm::{InterpretResult, VM};
use std::env;
use std::fs;
use std::io::{self, BufRead, Read, Write};
#[cfg(unix)]
use std::mem::MaybeUninit;

const BANNER: &str = include_str!("banner.txt");

pub fn run_main() -> Result<(), i32> {
    let args: Vec<String> = env::args().skip(1).collect();
    run_main_with_args(&args)
}

pub fn run_main_with_args(args: &[String]) -> Result<(), i32> {
    handle_args(args)
}

pub fn handle_args(args: &[String]) -> Result<(), i32> {
    handle_args_with_prompt(args, run_prompt)
}

pub fn handle_args_with_prompt<F>(args: &[String], prompt: F) -> Result<(), i32>
where
    F: FnOnce(),
{
    match args.len() {
        0 => {
            prompt();
            Ok(())
        }
        1 => run_file(&args[0]),
        _ => {
            eprintln!("Usage: oxython [script]");
            Err(64) // Standard exit code for command-line usage error
        }
    }
}

pub fn run_file(path: &str) -> Result<(), i32> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            if let Some(chunk) = Compiler::compile(&contents) {
                let mut vm = VM::new();
                vm.interpret(chunk);
                Ok(())
            } else {
                eprintln!("Compilation failed.");
                Err(65) // Standard exit code for data format error
            }
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            Err(74) // Standard exit code for I/O error
        }
    }
}

pub fn run_prompt() {
    #[cfg(unix)]
    {
        if unsafe { libc::isatty(libc::STDIN_FILENO) } != 0 {
            if let Err(err) = run_prompt_interactive() {
                eprintln!("Error: {}", err);
            }
            return;
        }
    }

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    let _ = run_prompt_with_io(&mut reader, &mut writer);
}

pub fn run_prompt_with_streams<R, W>(mut reader: R, mut writer: W) -> io::Result<W>
where
    R: BufRead,
    W: Write,
{
    run_prompt_with_io(&mut reader, &mut writer)?;
    Ok(writer)
}

pub fn run_prompt_with_io<R, W>(reader: &mut R, writer: &mut W) -> io::Result<()>
where
    R: BufRead,
    W: Write,
{
    let mut vm = VM::new();
    writeln!(writer, "{}", BANNER.trim_end())?;
    writeln!(writer)?;
    writeln!(writer, "Welcome to the oxython REPL! (Ctrl+D to exit)")?;
    loop {
        write!(writer, "> ")?;
        writer.flush()?;

        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        execute_line(&mut vm, &line, writer)?;
    }
    Ok(())
}

fn execute_line<W: Write>(vm: &mut VM, line: &str, writer: &mut W) -> io::Result<()> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    match Compiler::compile(trimmed) {
        Some(chunk) => {
            let has_expression_result = chunk.code.contains(&(OpCode::OpPop as u8));
            match vm.interpret(chunk) {
                InterpretResult::Ok => {
                    if has_expression_result {
                        let value = vm.last_popped_stack_elem();
                        if !matches!(&*value, ObjectType::Nil) {
                            writeln!(writer, "{}", &*value)?;
                        }
                    } else if let Some(value) = vm.peek_stack() {
                        if !matches!(&*value, ObjectType::Nil) {
                            writeln!(writer, "{}", &*value)?;
                        }
                    }
                }
                InterpretResult::RuntimeError => {
                    writeln!(writer, "Runtime error.")?;
                }
                InterpretResult::CompileError => {
                    writeln!(writer, "Compilation error.")?;
                }
            }
        }
        None => {
            writeln!(writer, "Compilation failed.")?;
        }
    }

    Ok(())
}

#[cfg(unix)]
fn run_prompt_interactive() -> io::Result<()> {
    let _raw = RawMode::new()?;

    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();

    writeln!(output, "{}", BANNER.trim_end())?;
    writeln!(output)?;
    writeln!(output, "Welcome to the oxython REPL! (Ctrl+D to exit)")?;

    let mut vm = VM::new();
    let mut history: Vec<String> = Vec::new();
    let mut history_pos: Option<usize> = None;
    let mut saved_input = String::new();
    let mut current_input = String::new();

    redraw_prompt(&mut output, &current_input)?;

    let mut buffer = [0u8; 1];
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            writeln!(output)?;
            break;
        }

        match buffer[0] {
            b'\n' | b'\r' => {
                writeln!(output)?;
                let command = current_input.trim().to_string();
                if !command.is_empty() {
                    if history.last() != Some(&command) {
                        history.push(command.clone());
                    }
                    execute_line(&mut vm, &command, &mut output)?;
                }
                current_input.clear();
                history_pos = None;
                saved_input.clear();
                redraw_prompt(&mut output, &current_input)?;
            }
            0x7f | 0x08 => {
                if !current_input.is_empty() {
                    current_input.pop();
                    history_pos = None;
                    saved_input.clear();
                    redraw_prompt(&mut output, &current_input)?;
                }
            }
            0x1b => {
                let mut seq = [0u8; 2];
                if input.read_exact(&mut seq).is_err() {
                    continue;
                }
                match seq {
                    [b'[', b'A'] => {
                        if history.is_empty() {
                            continue;
                        }
                        if history_pos.is_none() {
                            saved_input = current_input.clone();
                            history_pos = Some(history.len() - 1);
                        } else if let Some(pos) = history_pos {
                            if pos > 0 {
                                history_pos = Some(pos - 1);
                            }
                        }
                        if let Some(pos) = history_pos {
                            current_input = history[pos].clone();
                            redraw_prompt(&mut output, &current_input)?;
                        }
                    }
                    [b'[', b'B'] => {
                        if let Some(pos) = history_pos {
                            if pos + 1 < history.len() {
                                history_pos = Some(pos + 1);
                                current_input = history[pos + 1].clone();
                            } else {
                                history_pos = None;
                                current_input = saved_input.clone();
                            }
                            redraw_prompt(&mut output, &current_input)?;
                        }
                    }
                    _ => {}
                }
            }
            3 | 4 => {
                writeln!(output)?;
                break;
            }
            byte if byte.is_ascii_control() => {}
            byte => {
                current_input.push(byte as char);
                history_pos = None;
                saved_input.clear();
                redraw_prompt(&mut output, &current_input)?;
            }
        }
    }

    Ok(())
}

#[cfg(unix)]
fn redraw_prompt<W: Write>(writer: &mut W, buffer: &str) -> io::Result<()> {
    write!(writer, "\r> {}\x1b[K", buffer)?;
    writer.flush()
}

#[cfg(unix)]
struct RawMode {
    original: libc::termios,
}

#[cfg(unix)]
impl RawMode {
    fn new() -> io::Result<Self> {
        unsafe {
            let mut original = MaybeUninit::<libc::termios>::uninit();
            if libc::tcgetattr(libc::STDIN_FILENO, original.as_mut_ptr()) == -1 {
                return Err(io::Error::last_os_error());
            }
            let original = original.assume_init();

            let mut raw = original;
            raw.c_lflag &= !(libc::ICANON | libc::ECHO);
            raw.c_iflag &= !(libc::IXON | libc::ICRNL);
            raw.c_cc[libc::VMIN] = 1;
            raw.c_cc[libc::VTIME] = 0;

            if libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw) == -1 {
                return Err(io::Error::last_os_error());
            }

            Ok(Self { original })
        }
    }
}

#[cfg(unix)]
impl Drop for RawMode {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &self.original);
        }
    }
}
