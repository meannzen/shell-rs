#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub arguments: Vec<String>,
    pub input: Option<String>,
    pub output: Option<(String, i32)>,
}

#[derive(Debug)]
pub struct Pipeline {
    pub commands: Vec<Command>,
}
