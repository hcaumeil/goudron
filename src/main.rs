mod error;
mod lexer;
mod parser;
mod test;
mod vm;

use std::env::args;

fn usage() {
    println!("
goudron : lightweight api tester
Usage : goudron [option] file...
Option : -h, --help      Show this help.
         -b, --blocking  Stop the program after an route error.
         -f, --formated  Run script(s) and without any print, only a formated response (true or false).
         -s, --silent    Run script(s) with no route error print.
         -q, --quiet     Run script(s) without the print keyword.
    ")
}

fn main() {
    let mut arg: Vec<String> = args().collect();
    arg.remove(0);

    if arg.len() == 0 {
        usage();
        std::process::exit(1);
    }

    if arg.len() == 1 && (arg[0].as_str() == "-h" || arg[0].as_str() == "--help") {
        usage();
    } else {
        let mut blocking = false;
        let mut formated = false;
        let mut silent = false;
        let mut quiet = false;

        while arg.len() > 0 && arg[0].starts_with('-') {
            if arg[0] == String::from("-b") || arg[0] == String::from("--blocking") {
                arg.remove(0);
                blocking = true;
            } else if arg[0] == String::from("-f") || arg[0] == String::from("--formated") {
                arg.remove(0);
                formated = true;
                silent = true;
                quiet = true;
            } else if arg[0] == String::from("-s") || arg[0] == String::from("--silent") {
                arg.remove(0);
                silent = true;
            } else if arg[0] == String::from("-q") || arg[0] == String::from("--quiet") {
                arg.remove(0);
                quiet = true;
            } else {
                eprint!("error: {} : No such option\n", arg[0]);
                usage();
                std::process::exit(1);
            }
        }

        if arg.len() == 0 {
            eprint!("error: No input files\n");
            usage();
            std::process::exit(1);
        }

        let mut res: (usize, usize) = (0, 0); // (ok route, err route)

        for a in arg {
            let mut err = error::ErrorHandler::new(a.clone());

            match lexer::get_file_buf(a.as_str(), &mut err) {
                Some(b) => {
                    let t = lexer::Lexer::new(b, &mut err).get_tokens();
                    let inst = parser::Parser::new(t, &mut err).parse();
                    err.display();

                    // if inst[0] == vm::Inst::InstPush(String::from("https://httpbin.org/anything")) {
                    //     print!("1");
                    // }

                    // // if inst[1] == vm::Inst::InstPush(String::from("michel")) {
                    // //     print!("2");
                    // // }

                    // if inst[1] == vm::Inst::InstPush(String::from("200")) {
                    //     print!("3");
                    // }

                    // if inst[2] == vm::Inst::InstReq(String::from("POST"), true) {
                    //     print!("4");
                    // }

                    match vm::Vm::new(inst, silent, quiet, blocking).execute() {
                        Some(r) => {
                            res.0 += r.0;
                            res.1 += r.1
                        }
                        None => {
                            err.push(error::Error::ErrorWrongExec);
                            err.display();
                        }
                    }

                    if formated {
                        println!("{}", (res.1 == 0).to_string())
                    } else {
                        if res.0 + res.1 != 0 {
                            if res.1 == 0 {
                                println!("Done ! {} tests have been made with no errors.", res.0);
                            } else {
                                println!("Done ! {} tests have been made.", res.0 + res.1);
                                println!("{} were successful", res.0);
                                println!("{} came with errors", res.1);
                            }
                        }
                    }
                }
                None => err.display(),
            }
        }
    }
}
