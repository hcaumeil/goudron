#[cfg(test)]
mod test {
    use crate::error::Error::*;
    use crate::error::ErrorHandler;
    use crate::error::*;
    use crate::lexer::get_file_buf;
    use crate::lexer::Lexer;
    use crate::lexer::TokenSort::*;
    use crate::parser::Parser;
    use crate::vm;

    #[test]
    fn test_lexer() {
        let mut err = ErrorHandler::new(String::from("lex.goud"));
        let t =
            Lexer::new(get_file_buf("./test/lex.goud", &mut err).unwrap(), &mut err).get_tokens();

        assert_eq!(t.len(), 16);
        assert_eq!(t[0].sort, TokenId);
        assert_eq!(t[0].content, String::from("id"));
        assert_eq!(t[1].sort, TokenString);
        assert_eq!(t[1].content, String::from("string"));
        assert_eq!(t[2].sort, TokenNumber);
        assert_eq!(t[2].content, String::from("42"));
        assert_eq!(t[3].sort, TokenEq);
        assert_eq!(t[4].sort, TokenPlus);
        assert_eq!(t[5].sort, TokenQmark);
        assert_eq!(t[6].sort, TokenId);
        assert_eq!(t[6].content, String::from("printgetpost"));
        assert_eq!(t[7].sort, TokenId);
        assert_eq!(t[7].content, String::from("id"));
        assert_eq!(t[8].sort, TokenString);
        assert_eq!(t[8].content, String::from("string"));
        assert_eq!(t[9].sort, TokenNumber);
        assert_eq!(t[9].content, String::from("42"));
        assert_eq!(t[10].sort, TokenEq);
        assert_eq!(t[11].sort, TokenPlus);
        assert_eq!(t[12].sort, TokenQmark);
        assert_eq!(t[13].sort, TokenPrint);
        assert_eq!(t[14].sort, TokenGet);
        assert_eq!(t[15].sort, TokenPost);
    }

    #[test]
    fn test_err_1() {
        let mut err = ErrorHandler::new(String::from("err1.goud"));
        Parser::new(
            Lexer::new(
                get_file_buf("./test/err1.goud", &mut err).unwrap(),
                &mut err,
            )
            .get_tokens(),
            &mut err,
        )
        .parse();

        assert_eq!(err.errors.len(), 7);
        assert_eq!(err.errors[0], WarningEmptyString((4, 6)));
        assert_eq!(err.errors[1], WarningEscapeSeq((5, 8)));
        assert_eq!(
            err.errors[2],
            ErrorNullVar(
                Loc {
                    start: (1, 6),
                    end: (1, 13)
                },
                String::from("notinit")
            )
        );
        assert_eq!(
            err.errors[3],
            ErrorNoParse((2, 1), String::from("\"string\""))
        );
        assert_eq!(
            err.errors[4],
            ErrorUnxepectedToken(
                Loc {
                    start: (3, 6),
                    end: (3, 7)
                },
                String::from("a string or a variable"),
                (3, 5)
            )
        );
        assert_eq!(err.errors[5], ErrorNoParse((3, 6), String::from("?")));
        assert_eq!(
            err.errors[6],
            ErrorExpectedToken((6, 6), String::from("a string or a variable"))
        );
    }

    #[test]
    fn test_vm() {
        let mut err = ErrorHandler::new(String::from("vm.goud"));
        let prg = Parser::new(
            Lexer::new(get_file_buf("./test/vm.goud", &mut err).unwrap(), &mut err).get_tokens(),
            &mut err,
        )
        .parse();

        let res = vm::Vm::new(prg, true, true, false).execute();

        assert_eq!(res, Some((7, 0)));
    }
}
