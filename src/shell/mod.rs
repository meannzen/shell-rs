use std::{
    collections::HashMap,
    io::{self, Write},
};

#[derive(Debug)]
pub struct Shell {
    pub environment_var: HashMap<String, String>,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            environment_var: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        print!("$ ");
        io::stdout().flush().unwrap();
    }
}
