use std::io::{Stdin, ErrorKind, Result};
use super::logger::debug;
#[derive(Debug)]
pub struct GlobalInput<'a> {
    stdin: &'a mut Stdin,
}

impl <'a> GlobalInput<'a> {
    pub fn new(stdin: &'a mut Stdin) -> Self {
        GlobalInput { stdin }
    }

    pub fn read_line(&mut self) -> Result<String> {
        let mut buffer = String::new();
        loop {
            match self.stdin.read_line(&mut buffer) {
                Ok(_) => break,
                Err(e) if e.kind() != ErrorKind::Interrupted => continue,
                Err(e) => {
                    return Err(e);
                }
            }
        }
        debug!("the input line is {}", buffer.trim());
        let answer = buffer.trim().to_lowercase();
        Ok(answer)
    }
}
