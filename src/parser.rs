use crate::error::Error::*;
use crate::error::*;
use crate::lexer::TokenSort::*;
use crate::lexer::*;
use crate::vm::Inst::*;
use crate::vm::*;

pub struct Parser<'l> {
    cursor: usize,
    tokens: Vec<Token>,
    program: Vec<Inst>,
    var_checker: Vec<String>,
    err: &'l mut ErrorHandler,
}

impl<'l> Parser<'l> {
    pub fn new(tokens: Vec<Token>, err: &'l mut ErrorHandler) -> Self {
        Self {
            cursor: 0,
            tokens: tokens,
            program: Vec::new(),
            var_checker: Vec::new(),
            err: err,
        }
    }

    fn next(&mut self) {
        self.cursor += 1;
    }

    fn current_sort(&mut self) -> TokenSort {
        self.tokens[self.cursor].sort.to_owned()
    }

    fn current_value(&mut self) -> String {
        self.tokens[self.cursor].content.clone()
    }

    fn reach_end(&mut self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn check_near_end(&mut self, s: &str) -> bool {
        let res = self.cursor + 1 >= self.tokens.len();

        if res {
            self.add_err_exepected(s);
            self.next();
        }

        !res
    }

    fn add_inst(&mut self, sort: Inst) {
        self.program.push(sort);
    }

    fn add_inst_push(&mut self) {
        self.program
            .push(InstPush(self.tokens[self.cursor].content.clone()));
    }

    fn add_inst_load(&mut self) {
        self.program
            .push(InstLoad(self.tokens[self.cursor].content.clone()));
    }

    fn add_inst_gain(&mut self) {
        self.program
            .push(InstGain(self.tokens[self.cursor].content.clone()));
    }

    fn add_err(&mut self, e: Error) {
        self.err.push(e);
    }

    fn add_err_unexepected(&mut self, s: &str) {
        self.err.push(ErrorUnxepectedToken(
            self.tokens[self.cursor].loc.clone(),
            String::from(s),
            self.tokens[self.cursor - 1].loc.end,
        ));
    }

    fn add_err_exepected(&mut self, s: &str) {
        self.err.push(ErrorExpectedToken(
            self.tokens[self.cursor].loc.end,
            String::from(s),
        ));
    }

    fn push_var(&mut self, var: String) {
        if !self.var_checker.contains(&var) {
            self.var_checker.push(var);
        }
    }

    fn check_var(&mut self) -> bool {
        let s = self.current_value();
        self.var_checker.contains(&s)
    }

    pub fn parse_plus(&mut self) {
        if !self.reach_end() {
            if self.current_sort() == TokenPlus {
                if self.check_near_end("a string or a variable") {
                    self.next();

                    if self.parse_value() {
                        self.add_inst(InstPlus);
                    }
                }
            }
        }
    }

    fn parse_value(&mut self) -> bool {
        if !self.reach_end() {
            match self.current_sort() {
                TokenString => {
                    self.add_inst_push();
                    self.next();
                    self.parse_plus();
                    return true;
                }
                TokenId => {
                    if self.check_var() {
                        self.add_inst_gain();
                        self.next();
                        self.parse_plus();
                        return true;
                    } else {
                        self.add_err(ErrorNullVar(
                            self.tokens[self.cursor].loc.clone(),
                            self.tokens[self.cursor].content.clone(),
                        ));
                        self.next();
                    }
                }
                _ => self.add_err_unexepected("a string or a variable"),
            }
        }

        false
    }

    fn parse_id(&mut self) {
        if self.check_near_end("a token") {
            let var = self.current_value();
            self.next();

            if self.current_sort() == TokenEq {
                if self.check_near_end("a string or a variable") {
                    self.next();
                    if self.parse_value() {
                        self.push_var(var.clone());
                        self.add_inst(InstLoad(var));
                    }
                }
            } else {
                self.add_err_unexepected("an equal sign")
            }
        }
    }

    fn parse_print(&mut self) {
        if self.check_near_end("a string or a variable") {
            self.next();

            if self.parse_value() {
                self.add_inst(InstPrint);
            }
        }
    }

    pub fn parse_req(&mut self, method: &str) {
        if self.check_near_end("a string or a variable") {
            let mut body: bool = false;
            self.next();

            if self.parse_value() {
                if !self.reach_end() && self.current_sort() == TokenBody {
                    if self.check_near_end("a string or a variable") {
                        self.next();
                        if self.parse_value() {
                            body = true;
                        }
                    }
                }

                if !self.reach_end() {
                    if self.current_sort() == TokenNumber {
                        self.add_inst_push();
                        self.next();
                    } else {
                        self.add_inst(InstPush(String::from("200")));
                    }

                    if !self.reach_end() {
                        match self.current_sort() {
                            TokenEq => {
                                if self.check_near_end("a variable name") {
                                    self.next();

                                    if self.current_sort() == TokenId {
                                        let var = self.current_value();
                                        self.add_inst(InstReqandPush(String::from(method), body));
                                        self.add_inst_load();
                                        self.push_var(var);
                                        self.next();
                                    } else {
                                        self.add_err_unexepected("a variable name");
                                    }
                                }
                            }
                            TokenQmark => {
                                if self.check_near_end("a string or a variable") {
                                    self.next();

                                    if self.parse_value() {
                                        self.add_inst(InstReqandCompare(
                                            String::from(method),
                                            body,
                                        ));
                                    }
                                }
                            }
                            _ => {
                                self.add_inst(InstReq(String::from(method), body));
                            }
                        }
                    } else {
                        self.add_inst(InstReq(String::from(method), body));
                    }
                } else {
                    self.add_inst(InstPush(String::from("200")));
                    self.add_inst(InstReq(String::from(method), body));
                }
            }
        }
    }

    pub fn parse(&mut self) -> Vec<Inst> {
        while !self.reach_end() {
            match self.current_sort() {
                TokenId => self.parse_id(),
                TokenPrint => self.parse_print(),
                TokenGet => self.parse_req("GET"),
                TokenPost => self.parse_req("POST"),
                TokenPut => self.parse_req("PUT"),
                TokenDelete => self.parse_req("DELETE"),
                TokenString => {
                    self.add_err(ErrorNoParse(
                        self.tokens[self.cursor].loc.start,
                        String::from("\"") + self.tokens[self.cursor].content.as_str() + "\"",
                    ));
                    self.next();
                }
                _ => {
                    self.add_err(ErrorNoParse(
                        self.tokens[self.cursor].loc.start,
                        self.tokens[self.cursor].content.clone(),
                    ));
                    self.next();
                }
            }
        }

        self.program.to_owned()
    }
}
