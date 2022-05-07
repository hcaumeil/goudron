pub type Pos = (usize, usize); // (line, column)

#[derive(PartialEq, Clone, Debug)]
pub struct Loc {
    pub start: Pos,
    pub end: Pos,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Error {
    ErrorWrongPath,
    ErrorReadFile,
    ErrorReadline(usize),
    ErrorEmptyFile,
    ErrorUnclosedString(Pos),
    ErrorExpectedToken(Pos, String),
    ErrorUnxepectedToken(Loc, String, Pos),
    ErrorNullVar(Loc, String),
    ErrorNoParse(Pos, String),
    ErrorWrongExec,
    WarningEscapeSeq(Pos),
    WarningEmptyString(Pos),
}

#[derive(PartialEq, Clone, Debug)]
pub struct ErrorHandler {
    pub file: String,
    pub errors: Vec<Error>,
    pub trigger: bool,
}

impl ErrorHandler {
    pub fn new(file: String) -> Self {
        Self {
            file: file,
            errors: Vec::new(),
            trigger: false,
        }
    }

    pub fn display(&mut self) {
        if self.errors.len() != 0 {
            for e in self.errors.clone() {
                match e {
                    Error::ErrorWrongPath => {
                        eprintln!("error: {}: No such file or directory", self.file)
                    }
                    Error::ErrorReadFile => eprintln!("error: {}: Can't read the file", self.file),
                    Error::ErrorReadline(l) => {
                        eprintln!("error: {}: Can't read the file at line {}", self.file, l)
                    }
                    Error::ErrorEmptyFile => {
                        eprintln!("error: {}: No instructions were found", self.file)
                    }
                    Error::ErrorUnclosedString(p) => {
                        eprintln!("error: {}:{}:{} Unclose string", self.file, p.0, p.1)
                    }
                    Error::ErrorExpectedToken(p, s) => {
                        eprintln!("error: {}:{}:{} Expected token", self.file, p.0, p.1);
                        println!("note: {} is expexted", s);
                    }
                    Error::ErrorUnxepectedToken(l, s, p) => {
                        eprintln!(
                            "error: {}:{}:{} Unexpected token",
                            self.file, l.start.0, l.start.1
                        );
                        println!("note: {} is expexted at {}:{}:{}", s, self.file, p.0, p.1);
                    }
                    Error::ErrorNullVar(l, s) => eprintln!(
                        "error: {}:{}:{} Variable `{}` has no value",
                        self.file, l.start.0, l.start.1, s
                    ),
                    Error::ErrorNoParse(p, s) => eprintln!(
                        "error: {}:{}:{} Impossible to parse at token `{}`",
                        self.file, p.0, p.1, s
                    ),
                    Error::ErrorWrongExec => eprintln!(
                        "error: {}: Something went wrong with the execution",
                        self.file
                    ),
                    Error::WarningEscapeSeq(p) => eprintln!(
                        "warning: {}:{}:{} Unknow escape sequence",
                        self.file, p.0, p.1
                    ),
                    Error::WarningEmptyString(p) => {
                        eprintln!("warning: {}:{}:{} Empty string", self.file, p.0, p.1)
                    }
                }
            }

            if self.trigger {
                std::process::exit(1);
            } else {
                print!("\n");
            }
        }
    }

    pub fn push(&mut self, e: Error) {
        self.trigger = true;
        self.errors.push(e);
    }

    pub fn push_warning(&mut self, e: Error) {
        self.errors.push(e);
    }
}
