use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    CastOn,
    Knit,
    Purl,
    BindOff,
    Repeat,
    Ident(String),
    Number(i64),
    LBrace,
    RBrace,
    Semicolon,
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    EOF,
}

struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(src: &str) -> Self {
        Self {
            input: src.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    fn read_ident(&mut self, first: char) -> String {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.next();
            } else {
                break;
            }
        }
        s
    }

    fn read_number(&mut self, first: char) -> i64 {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.next();
            } else {
                break;
            }
        }
        s.parse().unwrap_or(0)
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        match self.next() {
            Some('{') => Token::LBrace,
            Some('}') => Token::RBrace,
            Some(';') => Token::Semicolon,
            Some('+') => Token::Plus,
            Some('-') => Token::Minus,
            Some('*') => Token::Star,
            Some('/') => Token::Slash,
            Some('=') => Token::Equal,
            Some(c) if c.is_ascii_alphabetic() => {
                let ident = self.read_ident(c);
                match ident.as_str() {
                    "cast_on" => Token::CastOn,
                    "knit" => Token::Knit,
                    "purl" => Token::Purl,
                    "bind_off" => Token::BindOff,
                    "repeat" => Token::Repeat,
                    other => Token::Ident(other.to_string()),
                }
            }
            Some(c) if c.is_ascii_digit() => Token::Number(self.read_number(c)),
            Some(_) => self.next_token(),
            None => Token::EOF,
        }
    }
}

#[derive(Debug)]
enum Expr {
    Number(i64),
    Var(String),
    Binary(Box<Expr>, char, Box<Expr>),
}

#[derive(Debug)]
enum Stmt {
    CastOn(String, Expr), // cast_on name = expr;
    Knit(String, Expr),   // knit name = expr;
    Purl(Expr),           // purl expr;
    Repeat(Expr, Vec<Stmt>),
    BindOff,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
    fn next(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        self.pos += 1;
        t
    }

    fn expect_ident(&mut self) -> String {
        match self.next() {
            Token::Ident(s) => s.clone(),
            other => panic!("Expected identifier, found: {:?}", other),
        }
    }

    fn expect_number_expr(&mut self) -> Expr {
        match self.next() {
            Token::Number(n) => Expr::Number(*n),
            other => panic!("Expected number, found: {:?}", other),
        }
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_add_sub()
    }

    fn parse_add_sub(&mut self) -> Expr {
        let mut node = self.parse_mul_div();
        loop {
            match self.peek() {
                Token::Plus => {
                    self.next();
                    let rhs = self.parse_mul_div();
                    node = Expr::Binary(Box::new(node), '+', Box::new(rhs));
                }
                Token::Minus => {
                    self.next();
                    let rhs = self.parse_mul_div();
                    node = Expr::Binary(Box::new(node), '-', Box::new(rhs));
                }
                _ => break,
            }
        }
        node
    }

    fn parse_mul_div(&mut self) -> Expr {
        let mut node = self.parse_term();
        loop {
            match self.peek() {
                Token::Star => {
                    self.next();
                    let rhs = self.parse_term();
                    node = Expr::Binary(Box::new(node), '*', Box::new(rhs));
                }
                Token::Slash => {
                    self.next();
                    let rhs = self.parse_term();
                    node = Expr::Binary(Box::new(node), '/', Box::new(rhs));
                }
                _ => break,
            }
        }
        node
    }

    fn parse_term(&mut self) -> Expr {
        match self.next() {
            Token::Number(n) => Expr::Number(*n),
            Token::Ident(name) => Expr::Var(name.clone()),
            other => panic!("Unexpected token in term: {:?}", other),
        }
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.peek() {
            Token::CastOn => {
                self.next();
                let name = self.expect_ident();
                match self.next() {
                    Token::Equal => {}
                    other => panic!("Expected = after identifier in cast_on, found {:?}", other),
                }
                let expr = self.parse_expr();
                match self.next() {
                    Token::Semicolon => {}
                    other => panic!("Expected ; after cast_on statement, found {:?}", other),
                }
                Some(Stmt::CastOn(name, expr))
            }
            Token::Knit => {
                self.next();
                let name = self.expect_ident();
                match self.next() {
                    Token::Equal => {}
                    other => panic!("Expected = after identifier in knit, found {:?}", other),
                }
                let expr = self.parse_expr();
                match self.next() {
                    Token::Semicolon => {}
                    other => panic!("Expected ; after knit statement, found {:?}", other),
                }
                Some(Stmt::Knit(name, expr))
            }
            Token::Purl => {
                self.next();
                let expr = self.parse_expr();
                match self.next() {
                    Token::Semicolon => {}
                    other => panic!("Expected ; after purl statement, found {:?}", other),
                }
                Some(Stmt::Purl(expr))
            }
            Token::Repeat => {
                self.next();
                let count = self.parse_expr();
                match self.next() {
                    Token::LBrace => {}
                    other => panic!("Expected '{{' after repeat count, found {:?}", other),
                }
                let mut body = Vec::new();
                while !matches!(self.peek(), Token::RBrace | Token::EOF) {
                    if let Some(s) = self.parse_stmt() {
                        body.push(s);
                    } else {
                        break;
                    }
                }
                match self.next() {
                    Token::RBrace => {}
                    other => panic!("Expected '}}' after repeat body, found {:?}", other),
                }
                Some(Stmt::Repeat(count, body))
            }
            Token::BindOff => {
                self.next();
                match self.next() {
                    Token::Semicolon => {}
                    other => panic!("Expected ; after bind_off, found {:?}", other),
                }
                Some(Stmt::BindOff)
            }
            Token::EOF => None,
            other => panic!("Unknown statement start: {:?}", other),
        }
    }

    fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Token::EOF) {
            if let Some(s) = self.parse_stmt() {
                stmts.push(s);
            } else {
                break;
            }
        }
        stmts
    }
}

struct Interpreter {
    vars: HashMap<String, i64>,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    fn eval_expr(&mut self, e: &Expr) -> i64 {
        match e {
            Expr::Number(n) => *n,
            Expr::Var(name) => *self.vars.get(name).unwrap_or(&0),
            Expr::Binary(lhs, op, rhs) => {
                let a = self.eval_expr(lhs);
                let b = self.eval_expr(rhs);
                match op {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    '/' => a / b,
                    _ => panic!("Unknown binary op: {}", op),
                }
            }
        }
    }

    fn exec_stmt(&mut self, s: &Stmt) -> bool {
        match s {
            Stmt::CastOn(name, expr) => {
                let v = self.eval_expr(expr);
                self.vars.insert(name.clone(), v);
                false
            }
            Stmt::Knit(name, expr) => {
                let v = self.eval_expr(expr);
                self.vars.insert(name.clone(), v);
                false
            }
            Stmt::Purl(expr) => {
                let v = self.eval_expr(expr);
                println!("{}", v);
                false
            }
            Stmt::Repeat(count_expr, body) => {
                let n = self.eval_expr(count_expr);
                for _ in 0..n {
                    for st in body {
                        if self.exec_stmt(st) {
                            return true;
                        }
                    }
                }
                false
            }
            Stmt::BindOff => true,
        }
    }

    fn run(&mut self, stmts: &[Stmt]) {
        for s in stmts {
            if self.exec_stmt(s) {
                break;
            }
        }
    }
}

fn lex_all(src: &str) -> Vec<Token> {
    let mut lx = Lexer::new(src);
    let mut tokens = Vec::new();
    loop {
        let t = lx.next_token();
        if t == Token::EOF {
            tokens.push(t);
            break;
        }
        tokens.push(t);
    }
    tokens
}

fn run_src(src: &str) {
    let tokens = lex_all(src);
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse();
    let mut interp = Interpreter::new();
    interp.run(&stmts);
}

fn repl() {
    let mut buf = String::new();
    let mut interp = Interpreter::new();
    loop {
        print!("knit> ");
        io::stdout().flush().unwrap();
        buf.clear();
        if io::stdin().read_line(&mut buf).is_err() {
            break;
        }
        let line = buf.trim();
        if line == "exit" || line == "quit" {
            break;
        }
        // try to parse a single statement
        let tokens = lex_all(line);
        let mut parser = Parser::new(tokens);
        match parser.parse_stmt() {
            Some(stmt) => {
                interp.exec_stmt(&stmt);
            }
            None => (),
        }
    }
}

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to a Knitlang source file to run
    file: Option<PathBuf>,

    /// Run REPL even if no file is provided
    #[arg(short, long)]
    repl: bool,

    /// Run a named example from the examples/ directory (e.g. --example hello)
    #[arg(long)]
    example: Option<String>,
}

fn main() {
    let args = <Args as clap::Parser>::try_parse().unwrap();

    if let Some(name) = args.example {
        let path = format!("examples/{}.knit", name);
        let src = fs::read_to_string(&path).expect("Failed to read example file");
        run_src(&src);
        return;
    }

    if let Some(path) = args.file {
        let src = fs::read_to_string(path).expect("Failed to read file");
        run_src(&src);
        return;
    }

    if args.repl || args.file.is_none() {
        println!("KNITLANG v2 - type 'exit' to quit. Try an example program as a .knit file and pass it as an argument.");
        repl();
    }
}
