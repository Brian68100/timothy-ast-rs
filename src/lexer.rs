use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::option::Option;
use std::collections::HashMap;
use crate::value::Value;
use crate::token::*;
use ordered_float::OrderedFloat;
use crate::gc::{Gc, NO_GC};

pub struct Lexer {
    fname: String,
    is_string_input: bool,
    buf: String,
    br: Option<BufReader<File>>,
    kwds: HashMap<String, TokenKind>,
    begin: usize,
    end: usize,
    last: bool,
    lineno: usize,
    colno: usize,
    pos: usize,
    nest: i32,
}

fn is_letter(c: char) -> bool {
    c.is_ascii_lowercase() ||
    c.is_ascii_uppercase() || c == '_'
}

fn is_num(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_letternum(c: char) -> bool {
    is_letter(c) || is_num(c)
}

fn get_keywords() -> HashMap<String, TokenKind> {
    HashMap::from([
        ("true".to_string(), TokenKind::TrueKw),
        ("false".to_string(), TokenKind::FalseKw),
        ("nil".to_string(), TokenKind::NilKw),
        ("let".to_string(), TokenKind::LetKw),
        ("fun".to_string(), TokenKind::FunKw),
        ("static".to_string(), TokenKind::StaticKw),
        ("macro".to_string(), TokenKind::MacroKw),
        ("class".to_string(), TokenKind::ClassKw),
        ("new".to_string(), TokenKind::NewKw),
        ("this".to_string(), TokenKind::ThisKw),
        ("super".to_string(), TokenKind::SuperKw),
        ("if".to_string(), TokenKind::IfKw),
        ("else".to_string(), TokenKind::ElseKw),
        ("while".to_string(), TokenKind::WhileKw),
        ("do".to_string(), TokenKind::DoKw),
        ("for".to_string(), TokenKind::ForKw),
        ("continue".to_string(), TokenKind::ContinueKw),
        ("break".to_string(), TokenKind::BreakKw),
        ("return".to_string(), TokenKind::ReturnKw),
    ])
}


impl Lexer {
    pub fn new_from_string(input: &mut String) -> Self {
        Self {
            fname: "<buf>".to_string(),
            is_string_input: true,
            buf: input.to_string(),
            br: None,
            kwds: get_keywords(),
            begin: 0,
            end: 0,
            last: false,
            lineno: 1,
            colno: 1,
            pos: 0,
            nest: 0,
        }
    }
    pub fn new_from_file(f: String) -> Option<Lexer> {
        let fresult: Result<File, std::io::Error> = File::open(f.clone());
        
        let mut l: Lexer = Lexer {
            fname: f,
            is_string_input: false,
            buf: String::new(),
            br: None,
            kwds: get_keywords(),
            begin: 0,
            end: 0,
            last: false,
            lineno: 1,
            colno: 1,
            pos: 0,
            nest: 0,
        };
        match fresult {
            Ok(file) => { 
                l.br = Some(BufReader::new(file));
                let br = l.br.as_mut().unwrap();
                let result = br.read_line(&mut l.buf);
                match result {
                    Ok(_) => {
                    },
                    Err(e) => { eprintln!("lexer error: {}", e);
                        return None;
                    },
                }
            },
            Err(e) => { eprintln!("lexer error: {}", e);
                        return None;
            },
        }
        Some(l)
    }

    fn at_end(&mut self) -> bool {
        if self.pos >= self.buf.len() && self.last {
            return true;
        }
        false 
    }

    pub fn file_name(&mut self) -> String {
        self.fname.to_string().clone()
    }
    fn peek(&mut self) -> char {
        if !self.at_end() &&  
            self.pos < self.buf.len() 
        {
            return self.buf.chars().nth(self.pos).unwrap();
        }
        '\0'
    }
    
    fn peek_next(&mut self) -> char {
        if self.pos + 1 < self.buf.len() {
            return self.buf.chars().nth(self.pos + 1).unwrap();
        }
        '\0'
    }

    fn advance(&mut self) -> char {
        if self.pos < self.buf.len() {
            self.pos += 1;
            self.colno += 1;
            return self.buf.chars().nth(self.pos - 1).unwrap();
        }
        
        self.end = self.pos;
        
        if !self.at_end() && !self.is_string_input {
            self.buf.clear();
            let br = self.br.as_mut().unwrap();
            let result = br.read_line(&mut self.buf); 
            match result {
                Ok(num) => {
                    if num == 0 {
                        self.last = true;
                        self.pos = 0;
                        return self.peek();
                    }
                    self.pos = 0;
                    self.colno += 1;
                    self.begin = 0;
                    self.end = 0;
                    
                    return self.peek();
                },
                    Err(e) => { eprintln!("lexer error: {}", e);
                    return '\0'; 
                },
            } 
        }
        '\0'
    } 
    fn match_char(&mut self, c : char) -> bool {
        if c == self.peek() {
            self.advance();
            return true;
        }
        false
    }
    fn read_identifier(&mut self) -> Token {
        let begin = self.pos - 1;
        let mut end = self.pos;
        loop {
            let c = self.peek();
            if !is_letternum(c) {
                break;
            }
            self.advance();
        }
        end = self.pos;
        println!("begin, end = ({}, {})", begin, end);
        println!("end - begin = {}", end - begin);
        let name: String = self.buf[begin..end].to_string();
        let value = self.kwds.get(&name);
        println!("name = {}", name);
                
        if let Some(val) = value {
            return Token::new(*val, TokenType::Keyword, name.clone(), self.lineno, self.colno, Value::Nil)
        } else {
            Token::new(TokenKind::Ident, TokenType::Ident, name.clone(), self.lineno, self.colno, Value::Nil)
        }
    }

    fn read_field(&mut self) -> Token {
        let begin = self.pos - 1;
        let mut end = self.pos;
        let c = self.peek();
        let mut kind = TokenKind::Field;
        if c == '#' {
            kind = TokenKind::StaticField;
            self.advance();
        }
        if is_letter(self.peek()) {
            self.advance();
            loop {
                let c = self.peek();
                if !is_letternum(c) {
                    break;
                }
                self.advance();
            }
            end = self.pos;
            return Token::new(kind, TokenType::Ident, self.buf[begin..end].to_string(), 
                self.lineno, self.colno, Value::Nil);
        }

        end = self.pos;
        Token::new(TokenKind::Hash, TokenType::Operator, self.buf[begin..end].to_string(), 
            self.lineno, self.colno, Value::Nil)
    }

    fn read_number(&mut self) -> Token {
        let begin = self.pos - 1; 
        loop { 
            if !is_num(self.peek()) {
                break;
            } 
            self.advance();
        }
        let mut end = self.pos;
        // Be sure not to advance if the next character is not a dot.
        if self.peek() != '.' || 
            (self.peek() == '.' && is_letter(self.peek_next()))
        {
            match self.buf[begin..end].to_string().parse() {
                Ok(num) => {
                    return Token::new(TokenKind::Int, TokenType::Other, self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Int(num));
                },
                Err(_) => {
                    return Token::new(TokenKind::Err, TokenType::Err, "lexer error: invalid integer token".to_string(),
                        self.lineno, self.colno, Value::Nil);
                },
            }
        }
        if is_num(self.peek_next()) {
            self.advance();
            loop {
                let c = self.peek();
                if !is_num(c) {
                    break;
                }
                self.advance();
            }
            end = self.pos;
        }
        match self.buf[begin..end].to_string().parse::<f64>() {
            Ok(num) => {
                Token::new(TokenKind::Float, TokenType::Other, self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Float(OrderedFloat(num)))
            },
            Err(_) => {
                Token::new(TokenKind::Err, TokenType::Err, "lexer error: invalid floating point token".to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
        }
    } 

    fn read_string(&mut self, gc: &Gc) -> Token {
        let begin = self.pos - 1;
        loop {
            if self.peek() == '\\' &&
                self.peek_next() == '"'
            {
                self.advance();
                self.advance();
                continue;
            } else if self.peek() == '"' {
                self.advance();
                return Token::new(TokenKind::String, TokenType::Other, self.buf[begin..self.pos].to_string(),
                    self.lineno, self.colno, Value::String(gc.manage(self.buf[begin + 1..self.pos - 1].to_string(), &NO_GC)));
            } else if self.peek() == '\n' {
                return Token::new(TokenKind::Err, TokenType::Err, "lexer error: unterminated string constant".to_string(),
                    self.lineno, self.colno, Value::Nil);
            }
            self.advance();
        }
    }

	fn skip_single_line_comment(&mut self) -> Option<bool> {
		self.nest = 0;
		if self.peek() == '/' {
			if self.peek_next() == '/' {
				self.advance();
				self.advance();
				loop {
					let chr = self.peek();
                    if chr == '\n' {
						self.advance();
                        break;
                    } else if chr == '\0' {
						if self.nest != 0 {
							println!("lexer error: {}: {}:{}: mismatched multiline comment delimiters", self.file_name(), self.lineno, self.colno);
                            return Some(false);
                        }
                        return None;
                    }
                    self.advance();
                }
			} else if self.peek_next() == '*' {
				self.advance();
                self.advance();
                self.nest += 1;
                if self.nest > 8 {
					println!("lexer error: {}: {}:{}: maximum multiline nesting depth exceeded; the limit is 8", self.file_name(), self.lineno, self.colno);
					return Some(false);
				}
			}
		} else if self.peek() == '*' && 
            self.peek_next() == '/' 
        {
			self.nest -= 1;
			if self.nest != 0 {
				println!("lexer error: {}: {}:{}: mismatched multiline comment delimiters", self.file_name(), self.lineno, self.colno);
                return Some(false);
            }
        }
		Some(true)
	}
    fn skip_whitespace(&mut self) -> bool {
		self.nest = 0;
        loop {
			self.skip_single_line_comment();
            match self.peek() {
                '\n' | '\t' | ' ' => {
                    if self.peek() == '\n' {
                        self.lineno += 1;
                        self.colno = 1;
                    }
                    self.advance();
                    loop {
                        let c = self.peek();
                        match c {
                            '\n' | '\t' | ' ' => {
                                if self.peek() == '\n' {
                                    self.lineno += 1;
                                    self.colno = 1;
                                }
                                self.advance();
                            },
                            '\0' => {
                                if self.advance() == '\0' {
                                    return true;
                                }
                            },
                            _ => {
                                //self.advance();
                                return true;
                            },
                        }
                    }
                },
                '\0' => {
                    if self.advance() == '\0' {
                        if self.nest != 0 {
                            println!("lexer error: {}: {}:{}: mismatched multiline comment delimiters", self.file_name(), self.lineno, self.colno - 1);
                            return false;
                        }
                        return true;
                    }
                    return true;
                },
                _ => {
                    if self.nest == 0 {
                        return true;
                    }
                },
            } 
        }
    } 

    pub fn scan(&mut self, gc: &Gc) -> Token {
        if !self.skip_whitespace() {
            return Token::new(TokenKind::Err, TokenType::Err, "".to_string(),
                self.lineno, self.colno, Value::Nil);
        }

        let ch: char = self.advance();
 
        if self.at_end() {
            return Token::new(TokenKind::Done, TokenType::None, "".to_string(),
                self.lineno, self.colno, Value::Nil);
        }
        if is_letter(ch) {
            return self.read_identifier();
        }
        if is_num(ch) {
            return self.read_number();
        }
        if ch == '#' {
            return self.read_field();
        }
        if ch == '"' {
            return self.read_string(gc);
        }

        let mut begin = 0; 
       
        if ch == '@' && self.peek() == '(' {
            self.advance();
            return Token::new(TokenKind::AnonClosure, TokenType::Operator,
                "@(".to_string(),
                self.lineno, self.colno, Value::Nil
            );
        }

        if ch != '\0' {
            begin = self.pos - 1;
        }

        match ch {
            '\0' => {
                Token::new(TokenKind::Done, TokenType::None,
                "".to_string(),
                self.lineno, self.colno, Value::Nil)
            },
            '(' => {
                let end = self.pos;
                Token::new(TokenKind::LParen, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            ')' => {
                let end = self.pos;
                Token::new(TokenKind::RParen, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '{' => {
                let end = self.pos;
                println!("end = {}", end);
                Token::new(TokenKind::LBrace, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '}' => {
                let end = self.pos;
                println!("(begin, end) = ({}, {})", begin, end);
                Token::new(TokenKind::RBrace, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '[' => {
                if self.match_char(']') {
                    if self.match_char('=') {
                        let end = self.pos;
                        return Token::new(TokenKind::IndexAssignMethod, TokenType::Operator, 
                            self.buf[begin..end].to_string(),
                            self.lineno, self.colno, Value::Nil);
                    }
                    let end = self.pos;
                    return Token::new(TokenKind::IndexMethod, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::LBracket, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            ']' => {
                
                let end = self.pos;
                Token::new(TokenKind::RBracket, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '?' => {
                let end = self.pos;
                Token::new(TokenKind::Question, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            ';' => {
                let end = self.pos;
                Token::new(TokenKind::Semicolon, TokenType::Other, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            ',' => {
                let end = self.pos;
                Token::new(TokenKind::Comma, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '.' => {
                let end = self.pos;
                Token::new(TokenKind::Dot, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            ':' => {
                let end = self.pos;
                Token::new(TokenKind::Colon, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '+' => {
                if self.match_char('=') {
                    let end = self.pos;
                    return Token::new(TokenKind::AddAssign, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::Add, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '-' => {
                if self.match_char('=') {
                    let end = self.pos;
                    return Token::new(TokenKind::SubtractAssign, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::Subtract, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '*' => {
                if self.match_char('*') {
                    if self.match_char('=') {
                        let end = self.pos;
                        return Token::new(TokenKind::PowerAssign, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                    }
                    let end = self.pos;
                    return Token::new(TokenKind::Power, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                if self.match_char('=') {
                    let end = self.pos;
                    return Token::new(TokenKind::MultiplyAssign, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::Multiply, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '/' => {
                if self.match_char('=') {
                    let end = self.pos;
                    return Token::new(TokenKind::DivideAssign, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::Divide, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '%' => {
                if self.match_char('=') {
                    let end = self.pos;
                    return Token::new(TokenKind::ModuloAssign, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::Modulo, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '!' => {
                if self.match_char('=') {
                    let end = self.pos;
                    return Token::new(TokenKind::NotEqual, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::LNot, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '=' => {
                if self.match_char('=') {
                    let end = self.pos;
                    return Token::new(TokenKind::Equal, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::Assign, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '<' => {
                if self.match_char('=') {
                    let end = self.pos;
                    return Token::new(TokenKind::LessEqual, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                if self.match_char('<') {
                    if self.match_char('=') {
                        let end = self.pos;
                        return Token::new(TokenKind::LShiftAssign, TokenType::Operator, 
                            self.buf[begin..end].to_string(),
                            self.lineno, self.colno, Value::Nil);
                    }
                    let end = self.pos;
                    return Token::new(TokenKind::LShift, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::Less, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '>' => {
                if self.match_char('=') {
                    let end = self.pos;
                    return Token::new(TokenKind::GreaterEqual, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                if self.match_char('<') {
                    if self.match_char('=') {
                        let end = self.pos;
                        return Token::new(TokenKind::RShiftAssign, TokenType::Operator, 
                            self.buf[begin..end].to_string(),
                            self.lineno, self.colno, Value::Nil);
                    }
                    let end = self.pos;
                    return Token::new(TokenKind::RShift, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::Greater, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '&' => {
                if self.match_char('&') {
                    let end = self.pos;
                    return Token::new(TokenKind::LAnd, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                if self.match_char('=') {
                    let end = self.pos;
                    return Token::new(TokenKind::AndAssign, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::And, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '|' => {
                if self.match_char('|') {
                    let end = self.pos;
                    return Token::new(TokenKind::LOr, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                if self.match_char('=') {
                    let end = self.pos;
                    return Token::new(TokenKind::OrAssign, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::Or, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            }, 
            '~' => {
                if self.match_char('=') {
                    let end = self.pos;
                    return Token::new(TokenKind::NotAssign, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::Not, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            '^' => {
                if self.match_char('=') {
                    let end = self.pos;
                    return Token::new(TokenKind::XOrAssign, TokenType::Operator, 
                        self.buf[begin..end].to_string(),
                        self.lineno, self.colno, Value::Nil);
                }
                let end = self.pos;
                Token::new(TokenKind::XOr, TokenType::Operator, 
                    self.buf[begin..end].to_string(),
                    self.lineno, self.colno, Value::Nil)
            },
            /*'"' => {
                self.read_string();
            },*/
            _ => { 
                Token::new(TokenKind::Err, TokenType::Err, "unknown token".to_string(),
                self.lineno, self.colno, Value::Nil)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_lexer_input() {
        test_input(&mut "".to_string());
    }
    #[test]
    fn test_lexer_arithmetic_exprs() {
        test_input(&mut "1".to_string());
        test_input(&mut "2 + 5".to_string());
        test_input(&mut "3 * 6 + 4".to_string());
        test_input(&mut "(17 - 6) + 10".to_string());
        test_input(&mut "10 % 2".to_string());
        test_input(&mut "10 ** 10".to_string());
    }
    #[test]
    fn test_lexer_let_decls() {
        test_input(&mut "let x = 100 / 20;".to_string());
        test_input(&mut "let nothing = nil;".to_string());
    }
    #[test]
    fn test_lexer_function_decls() {
        test_input(&mut "fun f(x) { return x; };".to_string());
        test_input(&mut "fun g(x, y) { return x * y; };".to_string());
        test_input(&mut "fun h(a,b) { a / b; };".to_string());
        test_input(&mut "fun i(x) {
                      x;
                    };".to_string());
    }
    #[test]
    fn test_lexer_function_call() {
        test_input(&mut "f(g, h);".to_string());
    }

    fn test_input(string: &mut String) {
        let mut gc = Gc::new();
        let mut l = Lexer::new_from_string(string);
        loop {
            let result = l.scan(&gc);
            if result.kind == TokenKind::Done {
                return;
            }
            if result.kind == TokenKind::Err {
                panic!("FAILED");
            }
        }
    }
}
