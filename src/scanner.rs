use regex::Regex;
use std::rc::{Rc};
use std::collections::VecDeque;
use std::fmt::Write;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Token {
    pub val: Rc<String>,
    pub token_name: &'static str,
    pub index: usize,
    pub length: usize,
    pub token_type: usize
}

impl<'a> fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.token_name == "keyword" {
            write!(f, "{}", self.val)
        } else {
            write!(f, "{}:{}", self.val, self.token_name)
        }
    }
}

impl Token {
    pub fn new(val: Rc<String>, token_name: &'static str, index: usize, length: usize, token_type: usize) -> Token {
        Token {
            val: val,
            token_name: token_name,
            index: index,
            length: length,
            token_type: token_type,
        }
    }

    pub fn get_blank() -> Token {
        Token {
            val: Rc::new("".to_owned()),
            token_name: "",
            index: 0,
            length: 0,
            token_type: 0,
        }
    }
}

pub fn parse_tokens(buffer: &String) -> Result<VecDeque<Token>, VecDeque<Token>> {
    lazy_static! {
        static ref KEYWORD1_RE: Regex = Regex::new(r"^((array)|(begin)|(boolean)|(break)|(do)|(else)|(end)|(enddo)|(endif)|(false)|(float)|(for)|(func)|(if)|(in)|(int)|(let)|(of)|(return)|(then)|(to)|(true)|(type)|(unit)|(var)|(while))$").unwrap();
        static ref KEYWORD2_RE: Regex = Regex::new(r"^(,|:|;|\(|\)|\[|\]|\{|\}|\.|\+|-|\*|/|=|<|>|<>|<=|>=|&|\||:=)$").unwrap();
        static ref ID_RE: Regex = Regex::new(r"^((_[A-Za-z_0-9]*([0-9A-Za-z])[A-Za-z_0-9]*)|(([A-Za-z])[0-9A-Za-z_]*))$").unwrap();
        static ref INTLIT_RE: Regex = Regex::new(r"^(\d*)$").unwrap();
        static ref FLOATLIT_RE: Regex = Regex::new(r"^(\d+\.\d*)$").unwrap();

        static ref TOKEN_REGEXES: Vec<&'static Regex> = vec![&FLOATLIT_RE, &INTLIT_RE, &ID_RE, &KEYWORD1_RE, &KEYWORD2_RE];
        static ref TOKEN_NAMES: Vec<&'static str> = vec!["floatlit", "intlit", "id", "keyword", "keyword"];
    }

    let (mut start, mut end) = (0, 1);
    let mut match_found: i32 = -1;
    let mut pre_match_found: i32 = -1;
    let mut need_keyword2 = false;

    let mut result: VecDeque<Token> = VecDeque::new();

    while end <= buffer.len() {
        if buffer[end - 1..end].eq(" ") {

            if pre_match_found < 0 {
                return Err(result);
            }


            if need_keyword2 && pre_match_found != 4 {
                result.pop_back();
                return Err(result);
            }

            if need_keyword2 && pre_match_found == 4 {
                need_keyword2 = false;
            }

            let token = Token {
                val: Rc::new(buffer[start..end - 1].to_owned()),
                token_name: TOKEN_NAMES[pre_match_found as usize],
                index: start, length: end - 1 - start,
                token_type: pre_match_found as usize
            };
            result.push_back(token);

            start = end;
            end += 1;
            pre_match_found = -1;

        }

        for (i, reg) in TOKEN_REGEXES.iter().enumerate() {
            if reg.is_match(&buffer[start..end]) {
                match_found = i as i32;
            }
        }

        if match_found == -1 && pre_match_found != -1 {
            if need_keyword2 && pre_match_found != 4 {
                result.pop_back();
                return Err(result);
            }

            if need_keyword2 && pre_match_found == 4 {
                need_keyword2 = false;
            }

            let token = Token {
                val: Rc::new(buffer[start..end - 1].to_owned()),
                token_name: TOKEN_NAMES[pre_match_found as usize],
                index: start, length: end - 1 - start,
                token_type: pre_match_found as usize
            };
            result.push_back(token);

            if pre_match_found <= 2 && pre_match_found >= 0 {
                need_keyword2 = true;
            }

            start = end - 1;
        } else {
            end += 1;
        }

        pre_match_found = match_found;
        match_found = -1;
    }

    if pre_match_found != -1 {
        if need_keyword2 && pre_match_found != 4 {
            result.pop_back();
            return Err(result);
        }

        let token = Token {
            val: Rc::new(buffer[start..end - 1].to_owned()),
            token_name: TOKEN_NAMES[pre_match_found as usize],
            index: start, length: end - 1 - start,
            token_type: pre_match_found as usize
        };
        result.push_back(token);
    }

    Ok(result)
}

/// Formats tokens into a human-friendly, space-delimited `String` list.
pub fn format_tokens(queue: &VecDeque<Token>) -> String {
    let mut output: String = String::new();
    let mut iter = queue.iter();
    if let Some(token) = iter.next() {
        write!(&mut output, "{}", token).unwrap();

        for token in iter {
            write!(&mut output, " {}", token).unwrap();
        }
    }

    output
}