use crossterm::style::Color;
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
        let operator_regex = Regex::new(r#"(?P<num>\d+(\.\d+)?([eE][+-]?\d+)?)|(?P<id>[a-zA-Z][a-zA-Z0-9_]*)|(?P<literal>"(?:\\.|[^"\\])*?")|(?P<op>(==|!=|<=|>=|<|>|[-+*/%&|^=!]=?|<<=?|>>=?|&&|\|\||[\(\)\{\}\[\]]))"#).unwrap();

        for cap in operator_regex.captures_iter(self.code.as_str()) {
            if let Some(m) = cap.name("num") {
                self.que.push_back(Token::Number(m.to_string()));
            } else if let Some(m) = cap.name("id") {
                self.que.push_back(Token::Identifier(m.to_string()));
            } else if let Some(m) = cap.name("literal") {
                self.que.push_back(Token::StringLiteral(String::from(
                    &m.to_string()[1..m.len() - 1],
                )));
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
    pub fn is_end(&self) -> bool {
        if self.lexer.peek().unwrap().str() == "EOF".to_string() {
            true
        } else {
            false
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

    pub fn program(&mut self) -> Result<Vec<AstNode>, String> {
        let mut res: Vec<AstNode> = vec![];
        while !self.is_end() {
            res.push(self.parse()?);
        }
        Ok(res)
    }
}

#[derive(Clone)]
pub enum Command {
    Paint(i64, i64, Color),
}

#[derive(Clone)]
enum Value {
    Num(f64),
    Str(String),
    Func(Vec<String>, Vec<AstNode>),
    Vector(Vec<Value>),
    Com(Command),
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
    pub fn push_env(&mut self) {
        self.stack.push(HashMap::new());
    }
    pub fn pop_env(&mut self) {
        self.stack.pop();
    }
}

pub struct Interpreter {
    environment: Environment,
    commands: Vec<Command>,
    program: Vec<AstNode>,
}
impl Interpreter {
    pub fn new(pro: Vec<AstNode>) -> Self {
        Self {
            environment: Environment::new(),
            commands: vec![],
            program: pro,
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

    fn to_number(&mut self, astnode: Value) -> Result<f64, String> {
        if let Value::Num(num) = astnode {
            Ok(num)
        } else {
            Err(format!(
                "the value was expected to be a number, but it is of another type."
            ))
        }
    }
    fn to_string(&mut self, astnode: Value) -> Result<String, String> {
        if let Value::Str(str) = astnode {
            Ok(str)
        } else {
            Err(format!(
                "the value was expected to be a string, but it is of another type."
            ))
        }
    }
    fn to_vector(&mut self, astnode: Value) -> Result<Vec<Value>, String> {
        if let Value::Vector(vec) = astnode {
            Ok(vec)
        } else {
            Err(format!(
                "the value was expected to be a vector, but it is of another type."
            ))
        }
    }

    fn eval(&mut self, astnode: AstNode) -> Result<Value, String> {
        match astnode {
            AstNode::Number(num) => Ok(Value::Num(num)),
            AstNode::Str(str) => Ok(Value::Str(str)),
            AstNode::Identifier(id) => Ok(self.environment.find(id)?),
            AstNode::List(list) => {
                let mut res = Value::Num(0.0);
                for astnode in list {
                    res = self.eval(astnode)?.clone();
                }
                Ok(res)
            }
            AstNode::Operater(op, children) => match op.as_str() {
                "+" => {
                    self.check_children_num(children.clone(), 2)?;
                    let val1: Value = self.eval(children[0].clone())?;
                    let val2: Value = self.eval(children[1].clone())?;
                    match self.to_number(val1.clone()) {
                        Ok(num) => Ok(Value::Num(num + self.to_number(val2)?)),
                        Err(_) => Ok(Value::Str(
                            self.to_string(val1.clone())?.clone()
                                + self.to_string(val2)?.clone().as_str(),
                        )),
                    }
                }
                "-" => {
                    self.check_children_num(children.clone(), 2)?;
                    let val1: Value = self.eval(children[0].clone())?;
                    let val2: Value = self.eval(children[1].clone())?;
                    Ok(Value::Num(self.to_number(val1)? - self.to_number(val2)?))
                }
                "*" => {
                    self.check_children_num(children.clone(), 2)?;
                    let val1: Value = self.eval(children[0].clone())?;
                    let val2: Value = self.eval(children[1].clone())?;
                    Ok(Value::Num(self.to_number(val1)? * self.to_number(val2)?))
                }
                "/" => {
                    self.check_children_num(children.clone(), 2)?;
                    let val1: Value = self.eval(children[0].clone())?;
                    let val2: Value = self.eval(children[1].clone())?;
                    Ok(Value::Num(self.to_number(val1)? / self.to_number(val2)?))
                }
                "%" => {
                    self.check_children_num(children.clone(), 2)?;
                    let val1: Value = self.eval(children[0].clone())?;
                    let val2: Value = self.eval(children[1].clone())?;
                    Ok(Value::Num(self.to_number(val1)? % self.to_number(val2)?))
                }
                "==" => {
                    self.check_children_num(children.clone(), 2)?;
                    let val1: Value = self.eval(children[0].clone())?;
                    let val2: Value = self.eval(children[1].clone())?;
                    match self.to_number(val1.clone()) {
                        Ok(num) => Ok(Value::Num(if num == self.to_number(val2)? {
                            1.0
                        } else {
                            0.0
                        })),
                        Err(_) => Ok(Value::Num(
                            if self.to_string(val1.clone())?.clone()
                                == self.to_string(val2)?.clone().as_str()
                            {
                                1.0
                            } else {
                                0.0
                            },
                        )),
                    }
                }
                "!=" => {
                    self.check_children_num(children.clone(), 2)?;
                    let val1: Value = self.eval(children[0].clone())?;
                    let val2: Value = self.eval(children[1].clone())?;
                    match self.to_number(val1.clone()) {
                        Ok(num) => Ok(Value::Num(if num != self.to_number(val2)? {
                            1.0
                        } else {
                            0.0
                        })),
                        Err(_) => Ok(Value::Num(
                            if self.to_string(val1.clone())?.clone()
                                != self.to_string(val2)?.clone().as_str()
                            {
                                1.0
                            } else {
                                0.0
                            },
                        )),
                    }
                }
                ">" => {
                    self.check_children_num(children.clone(), 2)?;
                    let val1 = self.eval(children[0].clone())?;
                    let val2 = self.eval(children[1].clone())?;
                    Ok(Value::Num(
                        if self.to_number(val1)? > self.to_number(val2)? {
                            1.0
                        } else {
                            0.0
                        },
                    ))
                }
                "<" => {
                    self.check_children_num(children.clone(), 2)?;
                    let val1 = self.eval(children[0].clone())?;
                    let val2 = self.eval(children[1].clone())?;
                    Ok(Value::Num(
                        if self.to_number(val1)? < self.to_number(val2)? {
                            1.0
                        } else {
                            0.0
                        },
                    ))
                }
                ">=" => {
                    self.check_children_num(children.clone(), 2)?;
                    let val1 = self.eval(children[0].clone())?;
                    let val2 = self.eval(children[1].clone())?;
                    Ok(Value::Num(
                        if self.to_number(val1)? >= self.to_number(val2)? {
                            1.0
                        } else {
                            0.0
                        },
                    ))
                }
                "<=" => {
                    self.check_children_num(children.clone(), 2)?;
                    let val1 = self.eval(children[0].clone())?;
                    let val2 = self.eval(children[1].clone())?;
                    Ok(Value::Num(
                        if self.to_number(val1)? <= self.to_number(val2)? {
                            1.0
                        } else {
                            0.0
                        },
                    ))
                }
                "set" => {
                    self.check_children_num(children.clone(), 2)?;
                    if let AstNode::Identifier(id) = &children[0] {
                        let value: Value = self.eval(children[1].clone())?;
                        self.environment.add(id.clone(), value.clone())?;
                        Ok(value)
                    } else {
                        Err(format!("the given must be an identifier."))
                    }
                }
                "const" => {
                    self.check_children_num(children.clone(), 2)?;
                    if let AstNode::Identifier(id) = &children[0] {
                        let value: Value = self.eval(children[1].clone())?;
                        self.environment.add_const(id.clone(), value.clone())?;
                        Ok(value)
                    } else {
                        Err(format!("the given must be an identifier."))
                    }
                }
                "func" => {
                    self.check_children_num(children.clone(), 2)?;
                    if let AstNode::IdList(li) = &children[0] {
                        let mut vec: Vec<AstNode> = vec![];
                        for i in 1..children.len() {
                            vec.push(children[i].clone());
                        }
                        Ok(Value::Func(li.clone(), vec))
                    } else {
                        Err(format!("you must provide a list of arguments."))
                    }
                }
                "if" => {
                    self.check_children_num(children.clone(), 3)?;
                    let exp = self.eval(children[0].clone())?.clone();
                    let block = if self.to_number(exp)? != 0.0 {
                        self.eval(children[1].clone())?.clone()
                    } else {
                        self.eval(children[2].clone())?.clone()
                    };
                    Ok(block)
                }
                "loop" => {
                    self.check_children_num(children.clone(), 2)?;
                    let mut res = Value::Num(0.0);
                    loop {
                        let exp = self.eval(children[0].clone())?.clone();
                        if self.to_number(exp)? == 0.0 {
                            break;
                        }
                        res = self.eval(children[1].clone())?.clone();
                    }
                    Ok(res)
                }
                "vec" => {
                    let mut vec: Vec<Value> = vec![];
                    for v in children {
                        vec.push(self.eval(v.clone())?.clone());
                    }
                    Ok(Value::Vector(vec.clone()))
                }
                "at" => {
                    self.check_children_num(children.clone(), 2)?;
                    let v: Value = self.eval(children[0].clone())?;
                    let i: Value = self.eval(children[1].clone())?;
                    Ok(self.to_vector(v)?.clone()[self.to_number(i)?.clone() as usize].clone())
                }
                /*
                "setat" => {
                    self.check_children_num(children.clone(), 3)?;
                    if let AstNode::Identifier(id) = &children[0] {
                        let index: Value = self.eval(children[1].clone())?;
                        let vec: Value = self.environment.find(id.clone())?;
                        Ok(value)
                    } else {
                        Err(format!("the given must be an identifier."))
                    }
                }
                */
                "paint" => {
                    self.check_children_num(children.clone(), 5)?;
                    let x: Value = self.eval(children[0].clone())?;
                    let y: Value = self.eval(children[1].clone())?;
                    let rgbvalue: (Value, Value, Value) = (
                        self.eval(children[2].clone())?,
                        self.eval(children[3].clone())?,
                        self.eval(children[4].clone())?,
                    );
                    let rgb: (f64, f64, f64) = (
                        self.to_number(rgbvalue.0)?,
                        self.to_number(rgbvalue.1)?,
                        self.to_number(rgbvalue.2)?,
                    );
                    let com = Command::Paint(
                        self.to_number(x)? as i64,
                        self.to_number(y)? as i64,
                        Color::Rgb {
                            r: rgb.0 as u8,
                            g: rgb.1 as u8,
                            b: rgb.2 as u8,
                        },
                    );
                    self.commands.push(com.clone());
                    Ok(Value::Com(com.clone()))
                }
                _ => {
                    if let Ok(val) = self.environment.find(op.clone()) {
                        if let Value::Func(args, code) = val {
                            self.environment.push_env();
                            self.check_children_num(children.clone(), args.len())?;
                            for i in 0..args.len() {
                                let ev = self.eval(children[i].clone())?;
                                self.environment.add(args[i].clone(), ev.clone())?;
                            }
                            let mut res: Value = Value::Num(0.0);
                            for c in code {
                                res = self.eval(c)?.clone();
                            }
                            self.environment.pop_env();
                            Ok(res)
                        } else {
                            Err(format!("variable '{}' is not function.", op.clone()))
                        }
                    } else {
                        Err(format!("invalid operater '{}'.", op))
                    }
                }
            },
            _ => Err(format!("invalid astnode.")),
        }
    }

    pub fn execute(&mut self) -> Result<Vec<Command>, String> {
        let mut res: Vec<Command> = vec![];
        self.commands = vec![];
        for ast in self.program.clone() {
            let val = self.eval(ast)?;
            match val {
                /*
                Value::Num(num) => {
                    res.push(num.to_string().clone());
                }
                Value::Str(str) => {
                    res.push(str.clone());
                }
                Value::Func(_, _) => {
                    res.push(format!("func"));
                }
                */
                Value::Vector(vec) => {
                    for e in vec {
                        if let Value::Com(com) = e {
                            res.push(com.clone())
                        }
                    }
                }
                Value::Com(command) => {
                    res.push(command.clone());
                }
                _ => {}
            }
        }
        Ok(self.commands.clone())
    }
}
