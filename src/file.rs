use failure::Error;

use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::errors::ParseError;
use crate::Interpreter;
use crate::log;

impl Interpreter {
    /// run each line of a file
    pub fn run_file<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path> + Debug,
    {
        let file = File::open(&path)?;
        let buf = BufReader::new(file);
        let mut lines = buf.lines().enumerate();

        let filename = path.as_ref()
            .file_name()
            .unwrap_or_else(|| OsStr::new("<unknown>"))
            .to_string_lossy();

        while let Some((linenum, Ok(line))) = lines.next() {
            if let Err(err) = self.run(line.as_str()) {
                match err.downcast::<ParseError>() {
                    Ok(ParseError::Empty) => continue,

                    Ok(err) => log::warn(format!(
                        "parsing error in {}:{}:\n  {}",
                        filename,
                        linenum + 1,
                        err
                    )),

                    Err(err) => log::warn(format!(
                        "runtime error in {}:{}:\n  {}",
                        filename,
                        linenum + 1,
                        err
                    )),
                }
            }
        }

        Ok(())
    }

    /// save the values in the Env to a runnable file
    pub fn save_env<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path> + Debug,
    {
        let file = File::create(path)?;
        let mut buf = BufWriter::new(file);
        writeln!(&mut buf, "; vim: set ft=scheme:")?;

        let env = self.env.clone();
        for (key, value) in &env.borrow().vars {
            writeln!(&mut buf, "(define {} {})", key, value.serialize())?;
        }

        Ok(())
    }
}
