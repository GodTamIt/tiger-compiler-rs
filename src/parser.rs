extern crate serde;
extern crate serde_json;

use log::Level;
use std::collections::{VecDeque, HashMap};
use std::fmt;
use std::fmt::Write;

use scanner::Token;

#[derive(Debug, Serialize, Deserialize)]
pub struct ParseTable {
    pub terminals: Vec<String>,
    pub table: Vec<Vec<usize>>
}

impl fmt::Display for ParseTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parse Table\nTerminals: {} - Table: {} x {}", self.terminals.len(), self.table.len(), self.table[0].len())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Grammar {
    pub nonterminals: Vec<String>,
    pub productions: Vec<Vec<String>>
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Grammar\nNon-Terminals: {} - Productions: {}", self.nonterminals.len(), self.productions.len())
    }
}


pub fn load_parse_table() -> Result<ParseTable, serde_json::Error> {
    serde_json::from_str(include_str!("../data/parsing_table.json"))
}

pub fn load_grammar() -> Result<Grammar, serde_json::Error> {
    serde_json::from_str(include_str!("../data/grammar.json"))
}

pub fn parse_input<'a>(grammar: &Grammar, table: &ParseTable, tokens: &mut VecDeque<Token<'a>>) -> Result<(Vec<String>, Vec<Token<'a>>), String> {
    // Build the reverse map for terminals
    let mut terminal_map: HashMap<&str, usize>= HashMap::new();
    for (i, terminal) in table.terminals.iter().enumerate() {
        terminal_map.insert(terminal, i);
        debug!("{}:{}", &terminal, &i);
    }

    let mut token = tokens.pop_front();
    let mut stack: VecDeque<String> = VecDeque::new();
    let mut ast: Vec<String> = Vec::new();
    let mut new_ast: Vec<Token<'a>> = Vec::new();
    let mut recurse_idx_stack: Vec<usize> = Vec::new();
    stack.push_front("1".to_string());

    loop {
        if stack.is_empty() && token.is_none() {
            break;
        } else if stack.is_empty() {
            return Err("unexpected values after end of program.".to_owned());
        } else if stack.front().unwrap().eq("@#") {
            // Found special token denoting the upwards traversal in the AST
            // Mark the end of an expanded non-terminal
            ast.push("@)".to_owned());
            new_ast.push(Token {val: "@)", token_name: "helper", index: 0, length: 0, token_type: 0});
            stack.pop_front();
            continue;
        }

        let non_terminal: usize = match stack.front().unwrap().parse::<usize>() {
            Ok(num) => num,
            Err(_e) => 0
        };

        let token_val = match token {
            Some(val) => val,
            None      => return Err("syntax error".to_owned())
        };

        print_debug_stack(&stack, &grammar.nonterminals);
        debug!("{}\n", token_val);

        if non_terminal == 0 {
            if (token_val.token_name.eq("keyword") && token_val.val.eq(stack.front().unwrap())) || token_val.token_name.eq(stack.front().unwrap()) {
                // success
                ast.push(get_token_ast_value(&token_val));
                new_ast.push(token_val);
                stack.pop_front();
                token = tokens.pop_front();
            } else {
                // error
                let mut err = String::new();
                write!(&mut err, "unexpected token '{}' found!", token_val.val).unwrap();
                return Err(err);
            }
        } else {
            let terminal_ndx: usize = match terminal_map.get(token_val.val) {
                Some(expr) => *expr,
                None => *terminal_map.get(token_val.token_name).unwrap(),
            };
            let row: &Vec<usize> = table.table.get(non_terminal).expect("Could not get table row");

            debug!("------------------");
            debug!("row: {:?}", row);
            debug!("Token: {}:{}", token_val.val, terminal_ndx);

            let production_no: usize = *row.get(terminal_ndx).expect("Could not get production number");
            let production = match grammar.productions.get(production_no) {
                Some(v) => v,
                None    => {
                    let mut err = String::new();
                    write!(&mut err, "unexpected token '{}' found!", token_val.val).unwrap();
                    return Err(err);
                }
            };

            debug!("{}:{} - {}:{} - {}:{}",
                &grammar.nonterminals.get(non_terminal).unwrap(),
                &non_terminal,
                &terminal_ndx,
                table.terminals.get(terminal_ndx).unwrap(),
                &production_no,
                &debug_production(&production, &grammar.nonterminals));
            debug!("------------------");


            stack.pop_front();

            // Push original productions onto the AST
            {
                let prod_name = grammar.nonterminals.get(non_terminal).unwrap();
                if prod_name.ends_with("^") {
                    // Push the index of the latest left-recursive production
                    recurse_idx_stack.push(ast.len());
                } else if prod_name.ends_with("^'") {
                    if production.is_empty() {
                        recurse_idx_stack.pop();
                    } else {
                        let recurse_idx = recurse_idx_stack.last_mut().unwrap();

                        // Insert at the recurse index to reintroduce left recursion
                        ast.insert(*recurse_idx, "@(".to_owned());
                        new_ast.insert(*recurse_idx, Token { val: "@(", token_name: "helper", index: 0, length: 0, token_type: 0});
                        ast.insert(*recurse_idx + 1, get_readable_production_name(prod_name));
                        let temp = get_readable_production_name(prod_name).as_str();
                        new_ast.insert(*recurse_idx + 1, Token {
                            val: temp,
                            token_name: "nonterminal",
                            index: 0,
                            length: 0,
                            token_type: 0
                        });
                        *recurse_idx += 2;

                        // Close the left recursive call inserted above
                        ast.push("@)".to_owned());
                        new_ast.push(Token {val: "@(", token_name: "helper", index: 0, length: 0, token_type: 0});
                    }
                }

                if !prod_name.ends_with("'") {
                    if !production.is_empty() {
                        // Push a special token to mark where the newly expanded production will end
                        stack.push_front("@#".to_owned());

                        // Mark the end of an expanded non-terminal
                        ast.push("@(".to_owned());
                        new_ast.push(Token {val: "@(", token_name: "helper", index: 0, length: 0, token_type: 0});
                    }

                    ast.push(get_readable_production_name(prod_name));
                    new_ast.push(Token {
                        val: &get_readable_production_name(prod_name),
                        token_name: "nonterminal",
                        index: 0,
                        length: 0,
                        token_type: 0
                    });
                }
            }

            // Push the expanded productions onto the stack (in reverse order)
            for rule in production.iter().rev() {
                stack.push_front(rule.to_owned());
            }

        }
    }

    Ok((ast, new_ast))
}

fn get_token_ast_value(token: &Token) -> String {
    match token.token_name {
        "keyword"   => token.val.to_owned(),
        "id"        => token.val.to_owned(),
        "intlit"    => token.val.to_owned(),
        "floatlit"  => token.val.to_owned(),
        _           => token.token_name.to_owned(),
    }
}

fn get_readable_production_name(name: &str) -> String {
    name.to_lowercase().replace("^", "").replace("'", "").to_owned()
}

fn debug_production(production: &Vec<String>, nonterminals: &Vec<String>) -> String {
    let mut output = String::new();
    for r in production {
        let non_terminal: usize = match r.parse::<usize>() {
            Ok(num) => num,
            Err(_e) => 0
        };
        if non_terminal != 0 {
            write!(output, "{} ", nonterminals.get(non_terminal).unwrap()).unwrap();
        } else {
            write!(output, "{} ", r).unwrap();
        }
    }
    output
}

fn print_debug_stack(stack: &VecDeque<String>, nonterminals: &Vec<String>) {
    if !log_enabled!(Level::Debug) {
        return;
    }

    let mut output = String::new();
    for e in stack {
        let non_terminal: usize = match e.parse::<usize>() {
            Ok(num) => num,
            Err(_e) => 0
        };
        if non_terminal != 0 {
            write!(output, "{}-", nonterminals.get(non_terminal).unwrap()).unwrap();
        } else {
            write!(output, "{}-", e).unwrap();
        }
    }
    write!(output, "$").unwrap();

    debug!("{}", output);
}

/// Formats the internally represented abstract syntax tree into a human-friendly `String`.
pub fn format_ast(ast: &Vec<String>) -> String {
    let mut output = String::new();
    let mut print_space = false;

    for symbol in ast {
        if print_space && !symbol.eq("@)") {
            write!(output, " ").unwrap();
        }

        print_space = !symbol.eq("@(");

        if symbol.starts_with("@") {
            write!(output, "{}", &symbol[1..]).unwrap();
        } else {
            write!(output, "{}", symbol).unwrap();
        }
    }

    return output;
}
