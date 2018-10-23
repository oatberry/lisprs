use failure::Error;

use crate::errors::ParseError;
use crate::values::Value::{self, *};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Item(String),
    LeftParen,
    RightParen,
}

/// parse a string of code into individual “bits” of syntax
pub fn tokenize(string: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut item = String::new();

    let mut escaped_state = false;
    let mut string_state = false;

    for c in string.chars() {
        if !string_state {
            match c {
                '(' => {
                    push_item(&mut item, &mut tokens);
                    tokens.push(Token::LeftParen);
                }

                ')' => {
                    push_item(&mut item, &mut tokens);
                    tokens.push(Token::RightParen);
                }

                ' ' => push_item(&mut item, &mut tokens),

                '"' => {
                    string_state = true;
                    item.push('"');
                }

                ';' => {
                    push_item(&mut item, &mut tokens);
                    return tokens;
                }

                _ => item.push(c),
            }
        } else if !escaped_state {
            match c {
                '\\' => escaped_state = true,

                '"' => {
                    string_state = false;
                    item.push('"');
                    push_item(&mut item, &mut tokens);
                }

                _ => item.push(c),
            }
        } else {
            escaped_state = false;
            item.push('\\');
            item.push(c);
        }
    }

    push_item(&mut item, &mut tokens);
    tokens
}

fn push_item(item: &mut String, tokens: &mut Vec<Token>) {
    if item.len() != 0 {
        tokens.push(Token::Item(item.clone()));
        item.clear();
    }
}

impl Value {
    /// parse a Vec of tokens into a structured s-expression
    pub fn from_tokens(tokens: &mut Vec<Token>) -> Result<Value, Error> {
        let token = tokens.remove(0);

        match token {
            Token::LeftParen => {
                let mut list: Vec<Value> = Vec::new();

                while tokens[0] != Token::RightParen {
                    list.push(Value::from_tokens(tokens)?);
                }

                tokens.remove(0);
                Ok(List(list))
            }

            Token::RightParen => Err(ParseError::ErroneousToken(")".to_string()))?,

            Token::Item(s) => {
                // handle quoted lists: '(<expr> <expr> ...)
                if s.as_str() == "'" {
                    if tokens.remove(0) != Token::LeftParen {
                        return Err(ParseError::ErroneousToken("'".to_string()))?;
                    }

                    let mut list: Vec<Value> = Vec::new();

                    while tokens[0] != Token::RightParen {
                        list.push(Value::from_tokens(tokens)?);
                    }

                    tokens.remove(0);
                    // this becomes: (quote (<expr> <expr> ...))
                    Ok(List(vec![Symbol("quote".to_owned()), List(list)]))
                } else {
                    Ok(Value::atomize(s))
                }
            }
        }
    }

    /// parse an item into an atom
    fn atomize(mut token: String) -> Value {
        if token.starts_with('"') && token.ends_with('"') && token.len() > 1 {
            token.pop();
            token.remove(0);
            Str(token)
        } else if let Ok(n) = token.parse::<f64>() {
            Number(n)
        } else if &token == "#t" {
            Bool(true)
        } else if &token == "#f" {
            Bool(false)
        } else if &token == "nil" {
            Nil
        } else {
            Symbol(token)
        }
    }
}
