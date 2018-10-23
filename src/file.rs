use failure::Error;

use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::Interpreter;
use crate::log;

impl Interpreter {
    /// run each line of a file
    pub fn run_file<P>(&self, path: P) -> Result<(), Error>
        where P: AsRef<Path> + Debug
    {
        log::info(format!("running {:?}...", path));

        let file = File::open(path)?;
        let buf = BufReader::new(file);
        let mut lines = buf.lines();

        while let Some(Ok(line)) = lines.next() {
            if let Err(err) = self.run(line.as_str()) {
                log::warn("an error ocurred:");
                log::warn(line);
                log::warn(err);
            }
        }

        log::info("run_file: done");
        Ok(())
    }

    /// save the values in the Env to a runnable file
    pub fn save_env<P>(&self, path: P) -> Result<(), Error>
        where P: AsRef<Path> + Debug
    {
        log::info(format!("saving current env to {:?}...", path));

        let file = File::create(path)?;
        let mut buf = BufWriter::new(file);
        writeln!(&mut buf, "; vim: set ft=scheme:")?;

        let env = self.env.clone();
        for (key, value) in &env.borrow().vars {
            writeln!(&mut buf, "(define {} {})", key, value.serialize())?;
        }

        log::info("save_env: done");
        Ok(())
    }
}
