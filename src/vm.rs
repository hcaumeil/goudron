use curl::easy::Easy;
use std::collections::HashMap;
use std::io::Read;
use std::process::exit;

#[derive(PartialEq, Clone, Debug)]
pub enum Inst {
    InstPush(String),
    InstLoad(String),
    InstGain(String),
    InstPlus,
    InstPrint,
    InstReq(String, bool),
    InstReqandPush(String, bool),
    InstReqandCompare(String, bool),
}

pub struct Vm {
    stack: Vec<String>,
    var: HashMap<String, String>,
    prg: Vec<Inst>,
    state: bool,
    ok: usize,
    err: usize,
    silent: bool,
    quiet: bool,
    blocking: bool,
}

impl Vm {
    pub fn new(prg: Vec<Inst>, silent: bool, quiet: bool, blocking: bool) -> Self {
        Self {
            stack: Vec::new(),
            var: HashMap::new(),
            prg: prg,
            state: true,
            ok: 0,
            err: 0,
            silent: silent,
            quiet: quiet,
            blocking: blocking,
        }
    }

    fn req(url: String, body: String, method: &str) -> Option<(String, String)> {
        let mut res = String::new();
        let code: String;
        let mut data = Vec::new();
        let mut handle = Easy::new();
        let mut b = Box::leak(body.into_boxed_str()).as_bytes();

        match handle.url(url.as_str()) {
            Ok(_) => {}
            Err(_) => return None,
        }

        match method {
            "GET" => match handle.get(true) {
                Ok(_) => {}
                Err(_) => return None,
            },
            "POST" => match handle.post(true) {
                Ok(_) => {}
                Err(_) => return None,
            },
            "PUT" => match handle.put(true) {
                Ok(_) => {}
                Err(_) => return None,
            },
            _ => {}
        }

        {
            let mut transfer = handle.transfer();

            match transfer.read_function(|into| Ok(b.read(into).unwrap())) {
                Ok(_) => {}
                Err(_) => return None,
            }

            match transfer.write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            }) {
                Ok(_) => {}
                Err(_) => return None,
            }

            match transfer.perform() {
                Ok(_) => {}
                Err(_) => return None,
            }
        }

        match handle.response_code() {
            Ok(c) => code = c.to_string(),
            Err(_) => return None,
        }

        for i in data {
            res.push(i as char);
        }

        Some((res, code))
    }

    pub fn execute(&mut self) -> Option<(usize, usize)> {
        let mut cursor = 0;

        while self.state && cursor < self.prg.len() {
            match &self.prg[cursor] {
                Inst::InstPush(s) => {
                    self.stack.push(s.to_string());
                }
                Inst::InstLoad(s) => {
                    self.var
                        .insert(s.to_string(), self.stack[self.stack.len() - 1].clone());
                }
                Inst::InstGain(s) => {
                    match self.var.get(s) {
                        Some(s) => {
                            self.stack.push(s.to_string());
                        }
                        None => {
                            // error var null
                        }
                    }
                }
                Inst::InstPlus => {
                    if self.stack.len() > 1 {
                        let s = self.stack[self.stack.len() - 2].clone()
                            + self.stack[self.stack.len() - 1].clone().as_str();
                        self.stack.pop();
                        self.stack.pop();
                        self.stack.push(s);
                    }
                }
                Inst::InstPrint => {
                    if !self.quiet {
                        println!("{}", self.stack[self.stack.len() - 1].clone());
                        self.stack.pop();
                    }
                }
                Inst::InstReq(m, b) => {
                    if self.stack.len() > 1 {
                        let mut body = String::from("");

                        let expected_code = self.stack[self.stack.len() - 1].clone();
                        self.stack.pop();

                        if *b {
                            body = self.stack[self.stack.len() - 1].clone();
                            self.stack.pop();
                        }

                        match Vm::req(self.stack[self.stack.len() - 1].clone(), body, m.as_str()) {
                            Some((_, code)) => {
                                if code == expected_code {
                                    self.stack.pop();
                                    self.ok += 1;
                                } else {
                                    if !self.silent {
                                        eprintln!(
                                            "route error: {} {} : Invalid respose code",
                                            m.as_str(),
                                            self.stack[self.stack.len() - 1].clone()
                                        );
                                    }
                                    self.stack.pop();
                                    self.err += 1;
                                }
                            }
                            None => {
                                if !self.silent {
                                    eprintln!(
                                        "route error: {} {} : Unable to make request",
                                        m.as_str(),
                                        self.stack[self.stack.len() - 1].clone()
                                    );
                                }
                                self.stack.pop();
                                self.err += 1;
                            }
                        }
                    } else {
                        self.err += 1;
                    }
                }
                Inst::InstReqandPush(m, b) => {
                    if self.stack.len() > 1 {
                        let mut body = String::from("");

                        let expected_code = self.stack[self.stack.len() - 1].clone();
                        self.stack.pop();

                        if *b {
                            body = self.stack[self.stack.len() - 1].clone();
                            self.stack.pop();
                        }

                        match Vm::req(self.stack[self.stack.len() - 1].clone(), body, m.as_str()) {
                            Some((response, code)) => {
                                if code == expected_code {
                                    self.stack.pop();
                                    self.ok += 1;
                                } else {
                                    if !self.silent {
                                        eprintln!(
                                            "route error: {} {} : Invalid respose code",
                                            m.as_str(),
                                            self.stack[self.stack.len() - 1].clone()
                                        );
                                    }
                                    self.stack.pop();
                                    self.err += 1;
                                }
                                self.stack.push(response);
                            }
                            None => {
                                if !self.silent {
                                    eprintln!(
                                        "route error: {} {} : Unable to make request",
                                        m.as_str(),
                                        self.stack[self.stack.len() - 1].clone()
                                    );
                                }
                                self.stack.pop();
                                self.err += 1;
                                self.stack.push(String::from(""));
                            }
                        }
                    } else {
                        self.err += 1;
                    }
                }
                Inst::InstReqandCompare(m, b) => {
                    if self.stack.len() > 2 {
                        let mut body = String::from("");

                        let expected_content = self.stack[self.stack.len() - 1].clone();
                        self.stack.pop();

                        let expected_code = self.stack[self.stack.len() - 1].clone();
                        self.stack.pop();

                        if *b {
                            body = self.stack[self.stack.len() - 1].clone();
                            self.stack.pop();
                        }

                        match Vm::req(self.stack[self.stack.len() - 1].clone(), body, m.as_str()) {
                            Some((response, code)) => {
                                if code == expected_code {
                                    if response == expected_content {
                                        self.stack.pop();
                                        self.ok += 1;
                                    } else {
                                        if !self.silent {
                                            eprintln!(
                                                "route error: {} {} : Invalid expected response",
                                                m.as_str(),
                                                self.stack[self.stack.len() - 1].clone()
                                            );
                                        }
                                        self.stack.pop();
                                        self.err += 1;
                                    }
                                } else {
                                    if !self.silent {
                                        eprintln!(
                                            "route error: {} {} : Invalid respose code",
                                            m.as_str(),
                                            self.stack[self.stack.len() - 1].clone()
                                        );
                                    }
                                    self.stack.pop();
                                    self.err += 1;
                                }
                            }
                            None => {
                                if !self.silent {
                                    eprintln!(
                                        "route error: {} {} : Unable to make request",
                                        m.as_str(),
                                        self.stack[self.stack.len() - 1].clone()
                                    );
                                }
                                self.stack.pop();
                                self.err += 1;
                            }
                        }
                    } else {
                        self.err += 1;
                    }
                }
            }
            cursor += 1;

            if self.blocking && self.err != 0 {
                exit(1);
            }
        }

        if self.prg.len() == cursor {
            Some((self.ok, self.err))
        } else {
            None
        }
    }
}
