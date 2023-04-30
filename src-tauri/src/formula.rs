use super::complex::Complex;
use nom::{branch, character, combinator, multi, number, IResult, bytes};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Operator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Exponentiation,
}
#[derive(Debug, Clone)]
enum Function {
    Cos,
    Sin,
    Ln,
    Sqrt,
    Tan,
    Step,
    // Delta,
    Exp,
}

#[derive(Debug)]
struct FunctionCall {
    function: Function,
    argument: Box<Node>
}
impl FunctionCall {
    fn evaluate(&self, input: Complex) -> Result<Complex, String> {
        let arg = self.argument.evaluate(input)?;
        match self.function {
            Function::Cos => Ok(arg.cos()),
            Function::Sin => Ok(arg.sin()),
            Function::Ln => Ok(arg.ln()),
            Function::Exp => Ok(arg.exp()),
            // Function::Delta => Ok(arg.delta()),
            Function::Sqrt => Ok(arg.sqrt()),
            Function::Tan => Ok(arg.tan()),
            Function::Step => Ok(arg.step()),
            _ => Err("Invalid function".into())
        }
    }

    fn evaluate_multivariable(&self, variables: &HashMap<char, Complex>) -> Result<Complex, String> {
        let arg = self.argument.evaluate_multivariable(variables)?;
        match self.function {
            Function::Cos => Ok(arg.cos()),
            Function::Sin => Ok(arg.sin()),
            Function::Ln => Ok(arg.ln()),
            Function::Exp => Ok(arg.exp()),
            // Function::Delta => Ok(arg.delta()),
            Function::Sqrt => Ok(arg.sqrt()),
            Function::Tan => Ok(arg.tan()),
            Function::Step => Ok(arg.step()),
            _ => Err("Invalid function".into())
        }
    }
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

    fn evaluate(&self, input: Complex) -> Result<Complex, String> {
        let left = self.left.evaluate(input)?;
        let right = self.right.evaluate(input)?;

        match self.operator {
            Operator::Addition => Ok(left + right),
            Operator::Subtraction => Ok(left - right),
            Operator::Multiplication => Ok(left * right),
            Operator::Division => {
                if right.is_zero() {
                    Err("Division by zero".into())
                } else {
                    Ok(left / right)
                }
            }
            Operator::Exponentiation => Ok(left.powf(&right)),
        }
    }

    fn evaluate_multivariable(
        &self,
        variables: &HashMap<char, Complex>,
    ) -> Result<Complex, String> {
        let left = self.left.evaluate_multivariable(variables)?;
        let right = self.right.evaluate_multivariable(variables)?;

        match self.operator {
            Operator::Addition => Ok(left + right),
            Operator::Subtraction => Ok(left - right),
            Operator::Multiplication => Ok(left * right),
            Operator::Division => {
                if right.is_zero() {
                    Err("Division by zero".into())
                } else {
                    Ok(left / right)
                }
            }
            Operator::Exponentiation => Ok(left.powf(&right)),
        }
    }
}

#[derive(Debug)]
enum Node {
    Value(f32),
    Variable(char),
    Operation(Operation),
    Function(FunctionCall),
}
impl Node {
    fn evaluate(&self, input: Complex) -> Result<Complex, String> {
        match &self {
            Node::Value(value) => Ok(Complex::from(*value)),
            Node::Variable(_) => Ok(input),
            Node::Operation(operation) => operation.evaluate(input),
            Node::Function(function) => function.evaluate(input),
        }
    }

    fn evaluate_multivariable(
        &self,
        variables: &HashMap<char, Complex>,
    ) -> Result<Complex, String> {
        match &self {
            Node::Value(value) => Ok(Complex::from(*value)),
            Node::Variable(variable) => match variables.get(variable) {
                None => Err(format!("Missing value for variable {}", variable)),
                Some(value) => Ok(*value),
            },
            Node::Operation(operation) => operation.evaluate_multivariable(variables),
            Node::Function(function) => function.evaluate_multivariable(variables),
        }
    }
}

#[derive(Debug)]
pub struct Formula {
    root: Node,
}
impl Formula {
    pub fn complex_phase(k: f32) -> Self {
        Self {
            root: Node::Operation(Operation {
                operator: Operator::Exponentiation,
                left: Box::new(Node::Value(std::f32::consts::E)),
                right: Box::new(Node::Operation(Operation {
                    operator: Operator::Multiplication,
                    left: Box::new(Node::Variable('i')),
                    right: Box::new(Node::Operation(Operation {
                        operator: Operator::Multiplication,
                        left: Box::new(Node::Value(k)),
                        right: Box::new(Node::Variable('x'))
                    })),
                })),
            }),
        }
    }

    pub fn new(formula: &str) -> Result<Self, String> {
        let (rest, tokens) =
            multi::many0(token)(formula).map_err(|_| String::from("Parsing error"))?;
        if rest != "" {
            return Err("Invalid character detected".into());
        }
        let root_scope = implicit_multiplication(create_global_scope(tokens)?);
        let root_node = create_sum(root_scope, true)?;
        Ok(Self { root: root_node })
    }

    pub fn get_vector(&self, start: f32, end: f32, length: u32) -> Vec<Complex> {
        let step = (end - start) / (length as f32);
        let mut values = vec![];
        for i in 0..length {
            let x = start + (i as f32) * step;
            let j = Complex::iunit();
            let mut variables = HashMap::from([
                ('x', Complex::from(x)),
                ('i', j)
            ]);
            values.push(self.evaluate_multivariable(&variables).unwrap_or(Complex::zero()))
        }
        values
    }

    pub fn adjoin(self, other: Formula, operator: Operator) -> Formula {
        Self {
            root: Node::Operation(Operation::new(operator, self.root, other.root)),
        }
    }

    pub fn evaluate(&self, input: Complex) -> Result<Complex, String> {
        self.root.evaluate(input)
    }

    pub fn evaluate_multivariable(
        &self,
        variables: &HashMap<char, Complex>,
    ) -> Result<Complex, String> {
        self.root.evaluate_multivariable(variables)
    }
}

#[derive(Debug, Clone)]
enum Token {
    Value(f32),
    Variable(char),
    Operator(Operator),
    Function(Function),
    OpeningBracket,
    ClosingBracket,
    Whitespace,
}

#[derive(Debug, Clone)]
enum ScopeElement {
    Token(Token),
    InnerScope(Vec<ScopeElement>),
}
impl ScopeElement {
    fn get_inner_scope(self) -> Result<Vec<Self>, String> {
        match self {
            ScopeElement::Token(_) => {
                Err("This scope element is a token, not an inner scope".into())
            }
            ScopeElement::InnerScope(inner_scope) => Ok(inner_scope),
        }
    }
}

fn function(input: &str) -> IResult<&str, Token> {
    let (rest, function) = branch::alt((
        bytes::complete::tag("cos"),
        bytes::complete::tag("sin"),
        bytes::complete::tag("sqrt"),
        bytes::complete::tag("ln"),
        bytes::complete::tag("log"),
        bytes::complete::tag("tan"),
        bytes::complete::tag("sen"),
        bytes::complete::tag("u"),
        // bytes::complete::tag("d"),
        bytes::complete::tag("exp")))(input)?;

    let function = match function {
        "cos" => Function::Cos,
        "sin" | "sen" => Function::Sin,
        "sqrt" => Function::Sqrt,
        "ln" | "log" => Function::Ln,
        "tan" => Function::Tan,
        "u" => Function::Step,
        // "d" => Function::Delta,
        "exp" => Function::Exp,
        _ => unreachable!()
    };

    Ok((rest, Token::Function(function)))
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
    branch::alt((function, bracket, operator, variable, value, whitespace))(input)
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
                if let ScopeElement::Token(Token::ClosingBracket) = scope_element {
                    return Err("Unmatched brackets".into());
                }
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
                if let ScopeElement::Token(Token::OpeningBracket) = scope_element {
                    depth += 1;
                }
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

/// Inserts implicit multiplication symbols
fn implicit_multiplication(scope: Vec<ScopeElement>) -> Vec<ScopeElement> {
    let mut new_scope = vec![];
    let mut last = None;

    for (i, scope_element) in scope.clone().into_iter().enumerate() {
        if let Some(last_scope_element) = last {
            if let ScopeElement::Token(Token::Value(_) | Token::Variable(_))
            | ScopeElement::InnerScope(_) = last_scope_element
            {
                if let ScopeElement::Token(Token::Value(_) | Token::Variable(_))
                | ScopeElement::InnerScope(_) = scope_element
                {
                    let mut next_is_exponentiation = false;
                    // yuck
                    for next_scope_element in &scope[i + 1..] {
                        if let ScopeElement::Token(Token::Operator(Operator::Exponentiation)) =
                            next_scope_element
                        {
                            next_is_exponentiation = true;
                            break;
                        } else if let ScopeElement::Token(Token::Whitespace) = next_scope_element {
                            continue;
                        } else {
                            break;
                        }
                    }
                    if next_is_exponentiation {
                        new_scope.push(last_scope_element);
                        new_scope.push(ScopeElement::Token(Token::Operator(
                            Operator::Multiplication,
                        )));
                        last = Some(scope_element);
                    } else {
                        let inner_scope = ScopeElement::InnerScope(vec![
                            last_scope_element,
                            ScopeElement::Token(Token::Operator(Operator::Multiplication)),
                            scope_element,
                        ]);
                        last = Some(inner_scope);
                    }
                } else {
                    if let ScopeElement::Token(Token::Whitespace) = last_scope_element {
                    
                    } else {
                        new_scope.push(last_scope_element);
                    }
                    last = Some(scope_element);
                }
            } else {
                if let ScopeElement::Token(Token::Whitespace) = last_scope_element {

                } else {
                    new_scope.push(last_scope_element);
                }
                last = Some(scope_element);
            }
        } else {
            last = Some(scope_element);
        }
    }

    if let Some(scope_element) = last {
        new_scope.push(scope_element);
    }

    new_scope

    /*
    new_scope.into_iter().filter(|scope_element|
        if let ScopeElement::Token(Token::Whitespace) = scope_element {
            false
        } else {
            true
        }
    ).collect::<Vec<ScopeElement>>()
    */
}

fn create_sum(mut scope: Vec<ScopeElement>, reverse: bool) -> Result<Node, String> {
    let mut left_scope = vec![];
    let mut right_scope = vec![];
    let mut operator = None;

    if reverse {
        scope.reverse();
    }
    for scope_element in scope {
        if operator.is_none() {
            match scope_element {
                ScopeElement::InnerScope(_) => right_scope.push(scope_element),
                ScopeElement::Token(ref token) => match token {
                    Token::Operator(Operator::Addition) => operator = Some(Operator::Addition),
                    Token::Operator(Operator::Subtraction) => {
                        operator = Some(Operator::Subtraction)
                    }
                    _ => right_scope.push(scope_element),
                },
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
            left_scope.push(ScopeElement::Token(Token::Value(0.0)));
        }
        let left = create_sum(left_scope, false)?;
        Ok(Node::Operation(Operation::new(
            operator.unwrap(),
            left,
            right,
        )))
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
                    Token::Operator(Operator::Multiplication) => {
                        operator = Some(Operator::Multiplication)
                    }
                    Token::Operator(Operator::Division) => operator = Some(Operator::Division),
                    _ => right_scope.push(scope_element),
                },
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
            Ok(Node::Operation(Operation::new(
                operator.unwrap(),
                left,
                right,
            )))
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
                    Token::Operator(Operator::Exponentiation) => {
                        operator = Some(Operator::Exponentiation)
                    }
                    _ => right_scope.push(scope_element),
                },
            }
        } else {
            left_scope.push(scope_element);
        }
    }
    let right = create_fn(right_scope)?;
    if operator.is_none() {
        Ok(right)
    } else {
        if left_scope.is_empty() {
            Err("Trailing operator".into())
        } else {
            let left = create_exp(left_scope)?;
            Ok(Node::Operation(Operation::new(
                operator.unwrap(),
                left,
                right,
            )))
        }
    }
}

fn create_fn(scope: Vec<ScopeElement>) -> Result<Node, String> {
    let mut argument = None;

    for scope_element in scope {
        match argument {
            None => argument = match scope_element {
                ScopeElement::InnerScope(inner_scope) => Some(create_sum(inner_scope, true)?),
                ScopeElement::Token(token) => match token {
                    Token::Value(value) => Some(Node::Value(value)),
                    Token::Variable(variable) => Some(Node::Variable(variable)),
                    _ => return Err("Unexpected error".into()),
                },
            },
            Some(argument_node) => match scope_element {
                ScopeElement::Token(Token::Function(function)) => {
                    argument = Some(Node::Function(FunctionCall {
                        function, argument: Box::new(argument_node) }))
                },
                _ => return Err("Expected function call".into()),
            }
        }
    }

    match argument {
        None => Err("Value or variable expected".into()),
        Some(argument) => Ok(argument)
    }
}
