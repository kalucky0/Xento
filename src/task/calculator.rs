use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

/// Binary operator
#[derive(Debug)]
pub enum Operator {
    Add,
    Substract,
    Multiply,
    Divide,
    Power,
    Modulo
}

/// A node in the tree
#[derive(Debug)]
pub enum Node {
    Value(f64),
    SubNode(Box<Node>),
    Binary(Operator, Box<Node>, Box<Node>),
}

/// parse a string into a node
pub fn parse(txt: &str) -> Option<Node> {
    let chars = txt.chars().filter(|c| *c != ' ').collect();
    parse_expression(&chars, 0).map(|(_, n)| n)
}

/// parse an expression into a node, keeping track of the position in the character vector
fn parse_expression(chars: &Vec<char>, pos: usize) -> Option<(usize, Node)> {
    match parse_start(chars, pos) {
        Some((new_pos, first)) => match parse_operator(chars, new_pos) {
            Some((new_pos2, op)) => {
                if let Some((new_pos3, second)) = parse_expression(chars, new_pos2) {
                    Some((new_pos3, combine(op, first, second)))
                } else {
                    None
                }
            }
            None => Some((new_pos, first)),
        },
        None => None,
    }
}

/// combine nodes to respect associativity rules
fn combine(op: Operator, first: Node, second: Node) -> Node {
    match second {
        Node::Binary(op2, v21, v22) => {
            if precedence(&op) >= precedence(&op2) {
                Node::Binary(op2, Box::new(combine(op, first, *v21)), v22)
            } else {
                Node::Binary(op, Box::new(first), Box::new(Node::Binary(op2, v21, v22)))
            }
        }
        _ => Node::Binary(op, Box::new(first), Box::new(second)),
    }
}

/// a precedence rank for operators
fn precedence(op: &Operator) -> usize {
    match op {
        Operator::Multiply | Operator::Divide => 2,
        _ => 1,
    }
}

/// try to parse from the start of an expression (either a parenthesis or a value)
fn parse_start(chars: &Vec<char>, pos: usize) -> Option<(usize, Node)> {
    match start_parenthesis(chars, pos) {
        Some(new_pos) => {
            let r = parse_expression(chars, new_pos);
            end_parenthesis(chars, r)
        }
        None => parse_value(chars, pos),
    }
}

/// match a starting parentheseis
fn start_parenthesis(chars: &Vec<char>, pos: usize) -> Option<usize> {
    if pos < chars.len() && chars[pos] == '(' {
        Some(pos + 1)
    } else {
        None
    }
}

/// match an end parenthesis, if successful will create a sub node contained the wrapped expression
fn end_parenthesis(chars: &Vec<char>, wrapped: Option<(usize, Node)>) -> Option<(usize, Node)> {
    match wrapped {
        Some((pos, node)) => {
            if pos < chars.len() && chars[pos] == ')' {
                Some((pos + 1, Node::SubNode(Box::new(node))))
            } else {
                None
            }
        }
        None => None,
    }
}

/// parse a value: an decimal with an optional minus sign
fn parse_value(chars: &Vec<char>, pos: usize) -> Option<(usize, Node)> {
    let mut new_pos = pos;
    if new_pos < chars.len() && chars[new_pos] == '-' {
        new_pos = new_pos + 1;
    }
    while new_pos < chars.len()
        && (chars[new_pos] == '.' || (chars[new_pos] >= '0' && chars[new_pos] <= '9'))
    {
        new_pos = new_pos + 1;
    }
    if new_pos > pos {
        if let Ok(v) = chars[pos..new_pos].iter().collect::<String>().parse() {
            Some((new_pos, Node::Value(v)))
        } else {
            None
        }
    } else {
        None
    }
}

/// parse an operator
fn parse_operator(chars: &Vec<char>, pos: usize) -> Option<(usize, Operator)> {
    if pos < chars.len() {
        let ops_with_char = vec![
            ('+', Operator::Add),
            ('-', Operator::Substract),
            ('*', Operator::Multiply),
            ('/', Operator::Divide),
            ('^', Operator::Power),
            ('%', Operator::Modulo),
        ];
        for (ch, op) in ops_with_char {
            if chars[pos] == ch {
                return Some((pos + 1, op));
            }
        }
    }
    None
}

/// eval a string
pub fn eval(txt: &str) -> f64 {
    match parse(txt) {
        Some(t) => eval_term(&t),
        None => panic!("Cannot parse {}", txt),
    }
}

/// raise to the power
fn pow(base: f64, exp: i64) -> f64 {
    let mut result = 1.0;
    for _ in 0..exp {
        result *= base;
    }
    result
}

/// find the remainder
fn modulo(base: f64, modulo: f64) -> f64 {
    let mut result = base;
    while result >= modulo {
        result -= modulo;
    }
    result
}

/// eval a term, recursively
fn eval_term(t: &Node) -> f64 {
    match t {
        Node::Value(v) => *v,
        Node::SubNode(t) => eval_term(t),
        Node::Binary(Operator::Add, t1, t2) => eval_term(t1) + eval_term(t2),
        Node::Binary(Operator::Substract, t1, t2) => eval_term(t1) - eval_term(t2),
        Node::Binary(Operator::Multiply, t1, t2) => eval_term(t1) * eval_term(t2),
        Node::Binary(Operator::Divide, t1, t2) => eval_term(t1) / eval_term(t2),
        Node::Binary(Operator::Power, t1, t2) => pow(eval_term(t1), eval_term(t2) as i64),
        Node::Binary(Operator::Modulo, t1, t2) => modulo(eval_term(t1), eval_term(t2)),
    }
}
