use crate::error::Error::*;
use crate::error::ErrorHandler;
use crate::error::Loc;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

#[derive(PartialEq, Clone, Debug)]
pub enum TokenSort {
    TokenId,
    TokenString,
    TokenNumber,
    TokenEq,
    TokenPlus,
    TokenQmark,
    TokenPrint,
    TokenBody,
    TokenGet,
    TokenPost,
    TokenPut,
    TokenDelete,
}

pub struct Token {
    pub sort: TokenSort,
    pub loc: Loc,
    pub content: String,
}

pub struct Lexer<'l> {
    pub file: BufReader<File>,
    pub c: char,
    pub line: String,
    pub cursor: usize,
    pub line_nb: usize,
    pub state: bool,
    pub err: &'l mut ErrorHandler,
}

impl<'l> Lexer<'l> {
    pub fn new(file: BufReader<File>, err: &'l mut ErrorHandler) -> Self {
        let mut res = Self {
            file: file,
            c: ' ',
            line: String::from(""),
            cursor: 0,
            line_nb: 0,
            state: true,
            err: err,
        };
        res.get_line();
        res
    }

    pub fn get_line(&mut self) {
        if self.state {
            let mut line = String::new();
            match self.file.read_line(&mut line) {
                Ok(n) => {
                    if n == (0 as usize) {
                        self.state = false;
                    } else {
                        self.cursor = 0;
                        self.line_nb = self.line_nb + 1;
                        self.line = line;
                    }
                }
                Err(_) => {
                    self.err.push(ErrorReadline(self.line_nb));
                    self.state = false;
                }
            };
        }
    }

    pub fn advance(&mut self) {
        if self.cursor == self.line.len() {
            self.get_line();
        }

        if self.state {
            match self.line.bytes().nth(self.cursor) {
                Some(c) => {
                    self.c = c as char;
                }
                None => {
                    self.err.push(ErrorReadline(self.line_nb));
                    self.state = false;
                }
            }
            self.cursor = self.cursor + 1;
        }
    }

    pub fn read_char(&mut self, ts: TokenSort) -> Token {
        let res = Token {
            sort: ts,
            loc: Loc {
                start: (self.line_nb, self.cursor),
                end: (self.line_nb, self.cursor + 1),
            },
            content: String::from(self.c),
        };
        self.advance();
        res
    }

    pub fn read_string(&mut self) -> Token {
        let start = (self.line_nb, self.cursor);
        let seq = self.c;
        self.advance();

        let content: String;
        let mut buffer: Vec<u8> = Vec::new();

        while (self.c != seq) && self.state {
            if self.c == '\\' {
                if self.cursor == self.line.len() {
                    self.err
                        .push_warning(WarningEscapeSeq((self.line_nb, self.cursor)));
                } else {
                    self.advance();
                    if self.c == 'n' {
                        buffer.push('\n' as u8);
                    } else if self.c == 't' {
                        buffer.push('\t' as u8);
                    } else if self.c == '\"' {
                        buffer.push('\"' as u8);
                    } else if self.c == '\\' {
                        buffer.push('\\' as u8);
                    } else {
                        self.err
                            .push_warning(WarningEscapeSeq((self.line_nb, self.cursor)));
                        buffer.push(self.c as u8);
                    }
                }
            } else {
                buffer.push(self.c as u8);
            }

            self.advance();
        }

        if !self.state || self.c != seq {
            self.err.push(ErrorUnclosedString(start));
        } else {
            self.advance();
        }

        match String::from_utf8(buffer) {
            Ok(r) => content = r,
            _ => {
                content = String::from("");
            }
        }

        if content.as_str() == "" {
            self.err.push_warning(WarningEmptyString(start));
        }

        Token {
            sort: TokenSort::TokenString,
            loc: Loc {
                start: start,
                end: (self.line_nb, self.cursor),
            },
            content: content,
        }
    }

    pub fn is_alpha(&mut self) -> bool {
        self.c.is_ascii_alphabetic()
    }

    pub fn read_id(&mut self) -> Token {
        let start = (self.line_nb, self.cursor);
        let mut content = String::new();

        while self.is_alpha() && self.state {
            content.push(self.c);
            self.advance();
        }

        let ts: TokenSort;

        if content.as_str() == "print" {
            ts = TokenSort::TokenPrint;
        } else if content.as_str() == "body" {
            ts = TokenSort::TokenBody;
        } else if content.as_str() == "get" {
            ts = TokenSort::TokenGet;
        } else if content.as_str() == "GET" {
            ts = TokenSort::TokenGet;
        } else if content.as_str() == "post" {
            ts = TokenSort::TokenPost;
        } else if content.as_str() == "POST" {
            ts = TokenSort::TokenPost;
        } else if content.as_str() == "put" {
            ts = TokenSort::TokenPut;
        } else if content.as_str() == "PUT" {
            ts = TokenSort::TokenPut;
        } else if content.as_str() == "delete" {
            ts = TokenSort::TokenDelete;
        } else if content.as_str() == "DELETE" {
            ts = TokenSort::TokenDelete;
        } else {
            ts = TokenSort::TokenId;
        }

        let res = Token {
            sort: ts,
            loc: Loc {
                start: start,
                end: (self.line_nb, self.cursor),
            },
            content: content,
        };
        res
    }

    pub fn is_digit(&mut self) -> bool {
        self.c.is_ascii_digit()
    }

    pub fn read_number(&mut self) -> Token {
        let start = (self.line_nb, self.cursor);
        let mut content = String::new();

        while self.is_digit() && self.state {
            content.push(self.c);
            self.advance();
        }

        let res = Token {
            sort: TokenSort::TokenNumber,
            loc: Loc {
                start: start,
                end: (self.line_nb, self.cursor),
            },
            content: content,
        };
        res
    }

    pub fn skip_space(&mut self) {
        while (self.c == ' ' || self.c == '\t') && self.state {
            self.advance();
        }
    }

    pub fn skip_line(&mut self) {
        self.get_line();
        self.advance();
    }

    pub fn get_tokens(&mut self) -> Vec<Token> {
        let mut res = Vec::new();

        while self.state {
            self.skip_space();

            if self.c == '#' {
                self.skip_line();
            } else if self.c == '=' {
                res.push(self.read_char(TokenSort::TokenEq));
            } else if self.c == '+' {
                res.push(self.read_char(TokenSort::TokenPlus));
            } else if self.c == '?' {
                res.push(self.read_char(TokenSort::TokenQmark));
            } else if self.c == '\"' || self.c == '\'' {
                res.push(self.read_string());
            } else if self.is_alpha() {
                res.push(self.read_id());
            } else if self.is_digit() {
                res.push(self.read_number());
            } else if self.c == '\n' {
                self.advance();
            } else {
                self.advance()
            }
        }

        if res.len() == 0 {
            self.err.push(ErrorEmptyFile);
        }

        res
    }
}

pub fn get_file_buf(file: &str, err: &mut ErrorHandler) -> Option<BufReader<File>> {
    if Path::new(&file).exists() {
        match File::open(file) {
            Ok(f) => Some(BufReader::new(f)),
            Err(_) => {
                err.push(ErrorReadFile);
                None
            }
        }
    } else {
        err.push(ErrorWrongPath);
        None
    }
}
