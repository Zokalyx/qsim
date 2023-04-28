use nom::{branch, character, combinator, multi, number, IResult};
use std::collections::HashMap;

#[derive(Debug)]
enum Operator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Exponentiation,
}

#[derive(Debug)]
struct Operation {
    operator: Operator,
    left: Box<Node>,
    right: Box<Node>,
}
impl Operation {
    fn new(operator: Operator, left: Node, right: Node) -> Self {
        Self {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn evaluate(&self, input: f32) -> Result<f32, String> {
        let left = self.left.evaluate(input)?;
        let right = self.right.evaluate(input)?;

        match self.operator {
            Operator::Addition => Ok(left + right),
            Operator::Subtraction => Ok(left - right),
            Operator::Multiplication => Ok(left * right),
            Operator::Division => {
                if right == 0.0 {
                    Err("Division by zero".into())
                } else {
                    Ok(left / right)
                }
            }
            Operator::Exponentiation => Ok(left.powf(right)),
        }
    }

    fn evaluate_multivariable(&self, variables: &HashMap<char, f32>) -> Result<f32, String> {
        let left = self.left.evaluate_multivariable(variables)?;
        let right = self.right.evaluate_multivariable(variables)?;

        match self.operator {
            Operator::Addition => Ok(left + right),
            Operator::Subtraction => Ok(left - right),
            Operator::Multiplication => Ok(left * right),
            Operator::Division => {
                if right == 0.0 {
                    Err("Division by zero".into())
                } else {
                    Ok(left / right)
                }
            }
            Operator::Exponentiation => Ok(left.powf(right)),
        }
    }
}

#[derive(Debug)]
enum Node {
    Value(f32),
    Variable(char),
    Operation(Operation),
}
impl Node {
    fn evaluate(&self, input: f32) -> Result<f32, String> {
        match &self {
            Node::Value(value) => Ok(*value),
            Node::Variable(_) => Ok(input),
            Node::Operation(operation) => operation.evaluate(input),
        }
    }

    fn evaluate_multivariable(&self, variables: &HashMap<char, f32>) -> Result<f32, String> {
        match &self {
            Node::Value(value) => Ok(*value),
            Node::Variable(variable) => match variables.get(variable) {
                None => Err(format!("Missing value for variable {}", variable)),
                Some(value) => Ok(*value),
            },
            Node::Operation(operation) => operation.evaluate_multivariable(variables),
        }
    }
}

#[derive(Debug)]
pub struct Formula {
    root: Node,
}
impl Formula {
    pub fn new(formula: &str) -> Result<Self, String> {
        let (rest, tokens) = multi::many0(token)(formula).map_err(|_| String::from("Parsing error"))?;
        if rest != "" {
            return Err("Invalid character detected".into());
        }
        let mut root_scope = create_global_scope(tokens)?;
        // Start from the end to respect multiple asymmetric operations (subtraction, division)
        root_scope.reverse();
        let root_node = create_sum(root_scope)?;  
        Ok(Self { root: root_node })
    }

    pub fn evaluate(&self, input: f32) -> Result<f32, String> {
        self.root.evaluate(input)
    }

    pub fn evaluate_multivariable(&self, variables: &HashMap<char, f32>) -> Result<f32, String> {
        self.root.evaluate_multivariable(variables)
    }
}

#[derive(Debug)]
enum Token {
    Value(f32),
    Variable(char),
    Operator(Operator),
    OpeningBracket,
    ClosingBracket,
    Whitespace,
}

#[derive(Debug)]
enum ScopeElement {
    Token(Token),
    InnerScope(Vec<ScopeElement>),
}
impl ScopeElement {
    fn get_inner_scope(self) -> Result<Vec<Self>, String> {
        match self {
            ScopeElement::Token(_) => Err("This scope element is a token, not an inner scope".into()),
            ScopeElement::InnerScope(inner_scope) => Ok(inner_scope),
        }
    }
}

fn bracket(input: &str) -> IResult<&str, Token> {
    let (rest, bracket) = character::complete::one_of("()")(input)?;
    let token = match bracket {
        '(' => Token::OpeningBracket,
        ')' => Token::ClosingBracket,
        _ => unreachable!(),
    };
    Ok((rest, token))
}
fn operator(input: &str) -> IResult<&str, Token> {
    let (rest, operator) = character::complete::one_of("+-*/^")(input)?;
    let operator = match operator {
        '+' => Operator::Addition,
        '-' => Operator::Subtraction,
        '*' => Operator::Multiplication,
        '/' => Operator::Division,
        '^' => Operator::Exponentiation,
        _ => unreachable!(),
    };
    Ok((rest, Token::Operator(operator)))
}
fn variable(input: &str) -> IResult<&str, Token> {
    let (rest, variable) =
        combinator::verify(character::complete::anychar, |c: &char| c.is_alphabetic())(input)?;
    Ok((rest, Token::Variable(variable)))
}
fn value(input: &str) -> IResult<&str, Token> {
    let (rest, value) = number::complete::float(input)?;
    Ok((rest, Token::Value(value)))
}
fn whitespace(input: &str) -> IResult<&str, Token> {
    let (rest, _) = character::complete::space1(input)?;
    Ok((rest, Token::Whitespace))
}

fn token(input: &str) -> IResult<&str, Token> {
    branch::alt((bracket, operator, variable, value, whitespace))(input)
}

fn create_global_scope(tokens: Vec<Token>) -> Result<Vec<ScopeElement>, String> {
    create_inner_scope(
        tokens
            .into_iter()
            .map(|token| ScopeElement::Token(token))
            .collect(),
    )?
    .get_inner_scope()
}

fn create_inner_scope(scope_elements: Vec<ScopeElement>) -> Result<ScopeElement, String> {
    let mut local_scope = vec![];
    let mut inner_scope = vec![];
    let mut depth = 0;

    for scope_element in scope_elements {
        if depth < 0 {
            return Err("Unmatched brackets".into());
        }
        if depth == 0 {
            if let ScopeElement::Token(Token::OpeningBracket) = scope_element {
                depth += 1;
            } else {
                local_scope.push(scope_element);
            }
        } else {
            if let ScopeElement::Token(Token::ClosingBracket) = scope_element {
                depth -= 1;
                if depth == 0 {
                    local_scope.push(create_inner_scope(inner_scope)?);
                    inner_scope = vec![];
                } else {
                    inner_scope.push(scope_element);
                }
            } else {
                inner_scope.push(scope_element);
            }
        }
    }

    if depth != 0 {
        Err("Unmatched brackets".into())
    } else {
        Ok(ScopeElement::InnerScope(local_scope))
    }
}

fn create_sum(scope: Vec<ScopeElement>) -> Result<Node, String> {
    let mut left_scope = vec![];
    let mut right_scope = vec![];
    let mut operator = None;

    for scope_element in scope {
        if operator.is_none() {
            match scope_element {
                ScopeElement::InnerScope(_) => right_scope.push(scope_element),
                ScopeElement::Token(ref token) => match token {
                    Token::Operator(Operator::Addition) => operator = Some(Operator::Addition),
                    Token::Operator(Operator::Subtraction) => operator = Some(Operator::Subtraction),
                    _ => right_scope.push(scope_element),
                }
            }
        } else {
            left_scope.push(scope_element);
        }
    }

    if right_scope.is_empty() {
        return Err("Trailing operator".into());
    }

    let right = create_mult(right_scope)?;
    if operator.is_none() {
        Ok(right)
    } else {
        if left_scope.is_empty() {
            Err("Trailing operator".into())
        } else {
            let left = create_sum(left_scope)?;
            Ok(Node::Operation(Operation::new(operator.unwrap(), left, right)))
        }
    }
}

fn create_mult(scope: Vec<ScopeElement>) -> Result<Node, String> {
    let mut left_scope = vec![];
    let mut right_scope = vec![];
    let mut operator = None;

    for scope_element in scope {
        if operator.is_none() {
            match scope_element {
                ScopeElement::InnerScope(_) => right_scope.push(scope_element),
                ScopeElement::Token(ref token) => match token {
                    Token::Operator(Operator::Multiplication) => operator = Some(Operator::Multiplication),
                    Token::Operator(Operator::Division) => operator = Some(Operator::Division),
                    _ => right_scope.push(scope_element),
                }
            }
        } else {
            left_scope.push(scope_element);
        }
    }

    if right_scope.is_empty() {
        return Err("Trailing operator".into());
    }

    let right = create_exp(right_scope)?;
    if operator.is_none() {
        Ok(right)
    } else {
        if left_scope.is_empty() {
            Err("Trailing operator".into())
        } else {
            let left = create_mult(left_scope)?;
            Ok(Node::Operation(Operation::new(operator.unwrap(), left, right)))
        }
    }
}

fn create_exp(scope: Vec<ScopeElement>) -> Result<Node, String> {
    let mut left_scope = vec![];
    let mut right_scope = vec![];
    let mut operator = None;

    for scope_element in scope {
        if operator.is_none() {
            match scope_element {
                ScopeElement::InnerScope(_) => right_scope.push(scope_element),
                ScopeElement::Token(ref token) => match token {
                    Token::Operator(Operator::Exponentiation) => operator = Some(Operator::Exponentiation),
                    _ => right_scope.push(scope_element),
                }
            }
        } else {
            left_scope.push(scope_element);
        }
    }

    let right = right_scope
        .into_iter()
        .filter_map(|scope_element| match scope_element {
            ScopeElement::InnerScope(inner_scope) => None,
            ScopeElement::Token(token) => match token {
                Token::Value(value) => Some(Node::Value(value)),
                Token::Variable(variable) => Some(Node::Variable(variable)),
                _ => None,
            }
        })
        .next()
        .ok_or(String::from("Missing value or variable"))?;
    if operator.is_none() {
        Ok(right)
    } else {
        if left_scope.is_empty() {
            Err("Trailing operator".into())
        } else {
            let left = create_exp(left_scope)?;
            Ok(Node::Operation(Operation::new(operator.unwrap(), left, right)))
        }
    }
}

/*fn pemdas(scope: Vec<ScopeElement>) -> Result<Node, String> {
    let mut scope = scope.into_iter();
    let last = scope.next().ok_or_else(|| String::from())?;

    while let Some(scope_element) = scope.next() {
        match last {
            ScopeElement::InnerScope(inner_scope) => panic!(),
            ScopeElement::Token(token) => match token {

            }
        }
    }

    Err(())
}
*/