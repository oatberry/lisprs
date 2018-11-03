mod log;

use itertools::join;
use std::path::PathBuf;

use lisprs::Interpreter;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "d", long = "debug")]
    debug: bool,

    #[structopt(name = "INITFILE", parse(from_os_str), help = "scheme file to run on startup")]
    initfile: Option<PathBuf>,
}

const HISTFILE: &str = ".lisprs_hist";

fn main() {
    let opt = Opt::from_args();
    if opt.debug {
        log::debug(format!("set options: {:?}", opt))
    }

    let interpreter = Interpreter::new();
    if let Some(initfile) = &opt.initfile {
        if let Err(why) = interpreter.run_file(initfile) {
            log::warn(why);
        }
    }

    let mut rl = Editor::<()>::new();
    if let Err(err) = rl.load_history(HISTFILE) {
        log::warn(format!("error opening history file: {}", err));
    }

    let prompt = format!("{}lisprs λ{} ", "\x1b[1;94m", log::RESET);

    loop {
        let input = rl.readline(&prompt);

        match input {
            Ok(line) => {
                if line.len() > 0 {
                    if line.starts_with(">") && line.len() > 1 {
                        println!("{}", command(&interpreter, &line[1..], &opt));
                    } else {
                        rl.add_history_entry(line.as_ref());
                        match interpreter.run(line) {
                            Ok(result) => println!("{}", result),
                            Err(err) => log::error(err),
                        }
                    }
                }
            }

            Err(ReadlineError::Interrupted) => {
                println!("^C");
                // break;
            }

            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }

            Err(err) => {
                log::error(err);
                break;
            }
        }
    }

    rl.save_history(HISTFILE).unwrap();
}

fn command(interpreter: &Interpreter, cmd: &str, opt: &Opt) -> String {
    match cmd {
        "env" => join(interpreter.env.borrow().vars.keys(), ", "),
        "save" => {
            if let Some(initfile) = &opt.initfile {
                match interpreter.save_env(initfile) {
                    Ok(_) => "".to_owned(),
                    Err(err) => {
                        log::warn(err);
                        "".to_owned()
                    }
                }
            } else {
                "no initfile set.".to_owned()
            }
        }
        _ => "invalid command".to_owned(),
    }
}
