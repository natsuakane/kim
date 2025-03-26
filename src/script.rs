use regex::Regex;
use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(Clone)]
pub enum Token {
    Number(String),
    StringLiteral(String),
    Identifier(String),
    EOF,
}
impl Token {
    pub fn str(&self) -> String {
        match &self {
            Token::Number(num) => num.clone(),
            Token::StringLiteral(lit) => lit.clone(),
            Token::Identifier(id) => id.clone(),
            Token::EOF => "EOF".to_string(),
        }
    }
}

pub struct Lexer {
    code: String,
    que: VecDeque<Token>,
}
impl Lexer {
    pub fn new(program: String) -> Self {
        Lexer {
            code: program,
            que: VecDeque::new(),
        }
    }
    pub fn read(&mut self) -> Option<Token> {
        self.que.pop_front()
    }
    pub fn peek(&self) -> Option<Token> {
        self.que.front().map(|t| t.clone())
    }
    pub fn lex(&mut self) {
        let re =
            Regex::new(r#"(?P<num>\d+(\.\d+)?)|(?P<id>[a-zA-Z][a-zA-Z0-9_]*)|(?P<literal>"(?:\\.|[^"\\])*?")|(?P<op>\S)"#)
                .unwrap();

        for cap in re.captures_iter(self.code.as_str()) {
            if let Some(m) = cap.name("num") {
                self.que.push_back(Token::Number(m.to_string()));
            } else if let Some(m) = cap.name("id") {
                self.que.push_back(Token::Identifier(m.to_string()));
            } else if let Some(m) = cap.name("literal") {
                self.que.push_back(Token::StringLiteral(m.to_string()));
            } else if let Some(m) = cap.name("op") {
                self.que.push_back(Token::Identifier(m.to_string()));
            }
        }

        self.que.push_back(Token::EOF);
    }
}

#[derive(Clone)]
pub enum AstNode {
    Number(f64),
    Str(String),
    List(Vec<AstNode>),
    IdList(Vec<String>),
    Operater(String, Vec<AstNode>),
    Identifier(String),
}
impl AstNode {
    pub fn print(&self) -> String {
        match self {
            AstNode::Number(num) => {
                format!("{}", num)
            }
            AstNode::Str(str) => format!("\"{}\"", str),
            AstNode::List(li) => {
                let mut res = String::from("'( ");
                for co in li {
                    res += &(co.print().clone());
                    res += " ";
                }
                res += ")";
                res
            }
            AstNode::IdList(li) => {
                let mut res = String::from("[ ");
                for co in li {
                    res += &co;
                    res += " ";
                }
                res += "]";
                res
            }
            AstNode::Operater(op, children) => {
                let mut res: String = format!("({} ", op);
                for ast in children.clone() {
                    res += &(ast.print());
                    res += " ";
                }
                res += ")";
                res
            }
            AstNode::Identifier(id) => id.clone(),
        }
    }
}

pub struct Parser {
    lexer: Lexer,
}
impl Parser {
    pub fn new(lex: Lexer) -> Self {
        Self { lexer: lex }
    }
    fn token(&mut self, id: &str) -> Result<(), String> {
        let token = self.lexer.read().unwrap();
        if let Token::Identifier(identifier) = token {
            if id == identifier.clone() {
                return Ok(());
            } else {
                return Err(format!(
                    "invalid token '{}', correct token is '{}'.",
                    identifier, id
                ));
            }
        } else {
            return Err(format!(
                "invalid token '{}', correct token is '{}'.",
                token.str(),
                id
            ));
        }
    }
    fn istoken(&mut self, t: &str) -> bool {
        match self.lexer.peek().unwrap() {
            Token::Identifier(op) => {
                return op == t;
            }
            Token::StringLiteral(_) => {
                return false;
            }
            Token::Number(_) => {
                return false;
            }
            Token::EOF => {
                return false;
            }
        }
    }
    fn get_id(&mut self) -> Result<String, String> {
        match self.lexer.read().unwrap() {
            Token::Identifier(id) => Ok(id),
            Token::StringLiteral(s) => Err(format!("String Literal \"{}\" is not identifier.", s)),
            Token::Number(n) => Err(format!("Number '{}' is not identifier.", n)),
            Token::EOF => Err(format!("'EOF' is not identifier.")),
        }
    }
    pub fn parse(&mut self) -> Result<AstNode, String> {
        println!("{}", self.lexer.que.len());
        if self.istoken("(") {
            self.token("(")?;
            let name = self.get_id()?;
            let mut children: Vec<AstNode> = vec![];
            while !self.istoken(")") {
                children.push(self.parse()?);
            }
            self.token(")")?;
            Ok(AstNode::Operater(name, children))
        } else if self.istoken("[") {
            self.token("[")?;
            let mut list: Vec<AstNode> = vec![];
            while !self.istoken("]") {
                list.push(self.parse()?);
            }
            self.token("]")?;
            Ok(AstNode::List(list))
        } else if self.istoken("{") {
            self.token("{")?;
            let mut list: Vec<String> = vec![];
            while !self.istoken("}") {
                list.push(self.get_id()?);
            }
            self.token("}")?;
            Ok(AstNode::IdList(list))
        } else {
            match self.lexer.read().unwrap() {
                Token::Number(n) => Ok(AstNode::Number(n.parse::<f64>().unwrap())),
                Token::StringLiteral(str) => Ok(AstNode::Str(str)),
                Token::Identifier(id) => Ok(AstNode::Identifier(id)),
                Token::EOF => Err(String::from("already EOF.")),
            }
        }
    }
}

#[derive(Clone)]
enum Value {
    Num(f64),
    Str(String),
    Vector(Vec<Value>),
}

struct Environment {
    stack: Vec<HashMap<String, (Value, bool)>>,
}
impl Environment {
    fn new() -> Self {
        Environment {
            stack: vec![HashMap::new()],
        }
    }
    pub fn find(&self, name: String) -> Result<Value, String> {
        for i in 0..self.stack.len() {
            if let Some(value) = self.stack[self.stack.len() - i - 1].get(&name) {
                return Ok(value.clone().0);
            }
        }
        Err(format!("Variable '{}' is not defined.", name))
    }
    pub fn add(&mut self, name: String, value: Value) -> Result<(), String> {
        let pos = self.stack.len() - 1;
        if let Some((_, b)) = self.stack[pos].get(&name) {
            if !b {
                Err(format!(
                    "The variable '{}' is a constant but you are trying to reassign it.",
                    name
                ))
            } else {
                self.stack.get_mut(pos).unwrap().insert(name, (value, true));
                Ok(())
            }
        } else {
            self.stack.get_mut(pos).unwrap().insert(name, (value, true));
            Ok(())
        }
    }
    pub fn add_const(&mut self, name: String, value: Value) -> Result<(), String> {
        let pos = self.stack.len() - 1;
        if let Some((_, b)) = self.stack[pos].get(&name) {
            if !b {
                Err(format!(
                    "The variable '{}' is a constant but you are trying to reassign it.",
                    name
                ))
            } else {
                self.stack
                    .get_mut(pos)
                    .unwrap()
                    .insert(name, (value, false));
                Ok(())
            }
        } else {
            self.stack
                .get_mut(pos)
                .unwrap()
                .insert(name, (value, false));
            Ok(())
        }
    }
    pub fn new_env(&mut self) {
        self.stack.push(HashMap::new());
    }
}

pub struct Interpreter {
    parser: Parser,
    environment: Environment,
}
impl Interpreter {
    pub fn new(par: Parser) -> Self {
        Self {
            parser: par,
            environment: Environment::new(),
        }
    }

    fn check_children_num(&self, children: Vec<AstNode>, num: usize) -> Result<(), String> {
        if children.len() == num {
            Ok(())
        } else {
            Err(format!(
                "expected {} arguments, but gave {} arguments.",
                num,
                children.len()
            ))
        }
    }

    fn to_number(&self, astnode: Value) -> Result<f64, String> {
        if let Value::Num(num) = astnode {
            Ok(num)
        } else {
            Err(format!(
                "the value was expected to be a number, but it is of another type."
            ))
        }
    }

    fn eval(&self, astnode: AstNode) -> Result<Value, String> {
        match astnode {
            AstNode::Number(num) => Ok(Value::Num(num)),
            AstNode::Str(str) => Ok(Value::Str(str)),
            AstNode::Identifier(id) => Ok(self.environment.find(id)?),
            AstNode::Operater(op, children) => match op.as_str() {
                "+" => {
                    self.check_children_num(children.clone(), 2)?;
                    Ok(Value::Num(
                        self.to_number(self.eval(children[0].clone())?)?
                            + self.to_number(self.eval(children[1].clone())?)?,
                    ))
                }
                _ => Err(format!("invalid operater '{}'.", op)),
            },
            _ => Err(format!("invalid astnode.")),
        }
    }

    pub fn execute(&mut self) -> Result<f64, String> {
        let pro = self.parser.parse()?;
        let val = self.eval(pro)?;
        if let Value::Num(num) = val {
            Ok(num)
        } else {
            Err(format!("something is wrong"))
        }
    }
}
