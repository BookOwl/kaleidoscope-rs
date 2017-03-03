use lexer;

/// Instead of creating a base class and multiple child classes,
/// we will use an enum to hold the different variants. This is much more Rusty
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Number(f64),
    Variable(String),
    Binary {
        op: char,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Box<Expr>>,
    }
}

// These structs hold the prototype and function ast nodes
#[derive(Debug, PartialEq, Clone)]
pub struct Prototype {
    pub name: String,
    pub args: Vec<String>,
}
impl Prototype {
    pub fn new(name: String, args: Vec<String>) -> Prototype {
        Prototype {
            name: name,
            args: args,
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub prototype: Prototype,
    pub body: Box<Expr>,
}
impl Function {
    pub fn new(prototype: Prototype, body: Box<Expr>) -> Function {
        Function {
            prototype: prototype,
            body: body,
        }
    }
}

// The Parser struct contains the lexer and has functions for parsing the token stream.
#[derive(Debug)]
pub struct Parser<'a> {
    lexer: lexer::Lexer<'a>,
    current: Option<lexer::Token>,
}
impl<'a> Parser<'a> {
    pub fn from_source(source: &'a str) -> Parser<'a> {
        Parser::from_lexer(lexer::Lexer::new(source))
    }
    pub fn from_lexer(mut lex: lexer::Lexer<'a>) -> Parser<'a> {
        let current = lex.next();
        Parser {
            lexer: lex,
            current: current,
        }
    }
    fn get_next_token(&mut self) {

        let tok = self.lexer.next();
        self.current = tok;
    }
    fn parse_number(&mut self) -> Result<Box<Expr>, String> {
        match self.current {
            Some(lexer::Token::Number(n)) => {
                self.get_next_token();
                Ok(Box::new(Expr::Number(n)))
            },
            ref x => Err(format!("Expected number, found {:?}", x))
        }
    }
    fn parse_paren_expr(&mut self) -> Result<Box<Expr>, String> {

        self.get_next_token();
        let v = self.parse_expression()?;
        match self.current {
            Some(lexer::Token::UnknownChar(')')) => Ok(v),
            ref x => Err(format!("Expected ), found {:?}", x))
        }
    }
    fn parse_identifier_expr(&mut self) -> Result<Box<Expr>, String> {

        let id = if let Some(lexer::Token::Identifier(ref s)) = self.current {
            s.clone()
        } else {
            return Err(format!("Expected identifier, found {:?}", self.current))
        };
        self.get_next_token();
        if Some(lexer::Token::UnknownChar('(')) == self.current {
            self.get_next_token();
            let mut args = Vec::new();
            loop {
                args.push(self.parse_expression()?);
                if Some(lexer::Token::UnknownChar(')')) == self.current {
                    break;
                }
                if Some(lexer::Token::UnknownChar(',')) != self.current {
                    return Err(format!("Expected \",\", found {:?}", self.current))
                }
                self.get_next_token();
            }
            self.get_next_token();
            Ok(Box::new(Expr::Call {
                name: id,
                args: args,
            }))
        } else {
            Ok(Box::new(Expr::Variable(id.clone())))
        }
    }
    fn parse_primary(&mut self) -> Result<Box<Expr>, String> {

        match self.current {
            Some(lexer::Token::Identifier(_)) => self.parse_identifier_expr(),
            Some(lexer::Token::Number(_)) => self.parse_number(),
            Some(lexer::Token::UnknownChar('(')) => self.parse_paren_expr(),
            _ => Err(format!("Unknown token {:?} when expecting an expression", self.current))
        }
    }
    fn parse_expression(&mut self) -> Result<Box<Expr>, String> {

        let lhs = self.parse_primary()?;
        self.parse_bin_op_rhs(0, lhs)
    }
    fn parse_bin_op_rhs(&mut self, prec: u32, mut lhs: Box<Expr>) -> Result<Box<Expr>, String> {
        loop {
            let op = match self.current {
                Some(lexer::Token::UnknownChar(c)) => c,
                _ => return Ok(lhs),
            };
            let tok_prec = match token_precedence(op) {
                Some(n) if n < prec => return Ok(lhs),
                None => return Ok(lhs),
                Some(n) => n,
            };
            self.get_next_token();
            let mut rhs = self.parse_primary()?;
            let next_prec = match self.current {
                Some(lexer::Token::UnknownChar(c)) => token_precedence(c),
                _ => None,
            };
            match next_prec {
                Some(n) if tok_prec < n => rhs = self.parse_bin_op_rhs(tok_prec + 1, rhs)?,
                //None => rhs = self.parse_bin_op_rhs(tok_prec + 1, rhs)?,
                _ => (),
            };
            lhs = Box::new(Expr::Binary {
                op: op,
                lhs: lhs,
                rhs: rhs,
            });
        }
    }
    pub fn parse_prototype(&mut self) -> Result<Prototype, String> {
        let name = match self.current {
            Some(lexer::Token::Identifier(ref name)) => name.clone(),
            ref x => return Err(format!("Expected identifier in prototype, found {:?}", x))
        };
        self.get_next_token();
        if self.current != Some(lexer::Token::UnknownChar('(')) {
            return Err(format!("Expected ( in prototype, found {:?}", self.current))
        }
        let mut arg_names = Vec::new();
        loop {
            self.get_next_token();
            match self.current {
                Some(lexer::Token::Identifier(ref arg_name)) => {
                    arg_names.push(arg_name.clone());
                },
                _ => break,
            }
        }
        if self.current != Some(lexer::Token::UnknownChar(')')) {
            return Err(format!("Expected ) in prototype, found {:?}", self.current))
        }
        self.get_next_token();
        Ok(Prototype::new(name, arg_names))
    }
    pub fn parse_definition(&mut self) -> Result<Function, String> {
        self.get_next_token(); // Eat "def"
        let proto = self.parse_prototype()?;
        let body = self.parse_expression()?;
        Ok(Function::new(proto, body))
    }
    pub fn parse_extern(&mut self) -> Result<Prototype, String> {
        self.get_next_token(); // eat "extern"
        self.parse_prototype()
    }
    pub fn parse_top_level_expr(&mut self) -> Result<Function, String> {
        let expr = self.parse_expression()?;
        let proto = Prototype::new(String::from(""), Vec::new());
        Ok(Function::new(proto, expr))
    }
}

fn token_precedence(tok: char) -> Option<u32> {
    match tok {
        '+' | '-' => Some(20),
        '<' => Some(10),
        '*' => Some(40),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_number_parsing() {
        let mut parser = Parser::from_source("1");
        let ast = parser.parse_expression().unwrap();
        assert_eq!(ast, Box::new(Expr::Number(1.0)));
        let mut parser = Parser::from_source("1234567890");
        let ast = parser.parse_expression().unwrap();
        assert_eq!(ast, Box::new(Expr::Number(1234567890.0)));
        let mut parser = Parser::from_source("3.14159");
        let ast = parser.parse_expression().unwrap();
        assert_eq!(ast, Box::new(Expr::Number(3.14159)));
        let mut parser = Parser::from_source("1.");
        let ast = parser.parse_expression().unwrap();
        assert_eq!(ast, Box::new(Expr::Number(1.0)));
        let mut parser = Parser::from_source(".1");
        let ast = parser.parse_expression().unwrap();
        assert_eq!(ast, Box::new(Expr::Number(0.1)));
    }
    #[test]
    fn test_basic_expression_parsing() {
        let mut parser = Parser::from_source("1 + 1");
        let ast = parser.parse_expression().unwrap();
        assert_eq!(ast, Box::new(Expr::Binary {
            op: '+',
            lhs: Box::new(Expr::Number(1.0)),
            rhs: Box::new(Expr::Number(1.0)),
        }))
    }
    #[test]
    fn test_complicated_expression_parsing() {
        let mut parser = Parser::from_source("1 + 2 * 3 - 2");
        let got = parser.parse_expression().unwrap();
        let expected = Box::new(Expr::Binary {
            op: '-',
            lhs: Box::new(Expr::Binary {
                op: '+',
                lhs: Box::new(Expr::Number(1.0)),
                rhs: Box::new(Expr::Binary {
                    op: '*',
                    lhs: Box::new(Expr::Number(2.0)),
                    rhs: Box::new(Expr::Number(3.0)),
                }),
            }),
            rhs: Box::new(Expr::Number(2.0)),
        });
        assert_eq!(got, expected)
    }
    #[test]
    fn test_prototype_parsing() {
        let mut parser = Parser::from_source("foo()");
        let got = parser.parse_prototype().unwrap();
        let expected = Prototype::new(String::from("foo"), vec![]);
        assert_eq!(got, expected);
        let mut parser = Parser::from_source("bar(a)");
        let got = parser.parse_prototype().unwrap();
        let expected = Prototype::new(String::from("bar"), vec![String::from("a")]);
        assert_eq!(got, expected);
        let mut parser = Parser::from_source("bar(a b c)");
        let got = parser.parse_prototype().unwrap();
        let expected = Prototype::new(String::from("bar"), vec![String::from("a"), String::from("b"), String::from("c")]);
        assert_eq!(got, expected);
    }
    #[test]
    fn test_function_definition_parsing() {
        let mut parser = Parser::from_source("def foo() 1 + 1");
        let got = parser.parse_definition().unwrap();
        let expected = Function::new(Prototype::new(String::from("foo"), vec![]),
                                     Box::new(Expr::Binary {
                                         op: '+',
                                         lhs: Box::new(Expr::Number(1.0)),
                                         rhs: Box::new(Expr::Number(1.0)),
                                     })
        );
        assert_eq!(got, expected);
    }
    #[test]
    fn test_extern_parsing() {
        let mut parser = Parser::from_source("extern sin(a)");
        let got = parser.parse_extern().unwrap();
        let expected = Prototype::new(String::from("sin"), vec![String::from("a")]);
        assert_eq!(got, expected);
    }
}
