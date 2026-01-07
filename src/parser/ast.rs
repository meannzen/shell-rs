#[derive(Debug, Clone)]
pub struct Redirection {
    pub path: String,
    pub fd: i32,
    pub append: bool,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub arguments: Vec<String>,
    pub input: Option<String>,
    pub outputs: Vec<Redirection>,
}

#[derive(Debug)]
pub struct Pipeline {
    pub commands: Vec<Command>,
}
