use std::{collections::HashMap, convert::TryInto};
use regex::Regex;
use rand::Rng;


static ANGLE_FACTOR: f64 = std::f64::consts::PI / 180.0;

#[derive(Debug)]
enum OperationType {
	Add,
	Subtract,
	Multiply,
	Divide,
	Negate,
	Invert,
	Ternary,
	And,
	Or,
	Smaller,
	SmallerEqual,
	Larger,
	LargerEqual,
	Equal,
	Unequal,
	NullCoalescing,
	Abs,
	Sin,
	Cos,
	Exp,
	Pow,
	Sqrt,
	Ln,
	Random,
	Ceil,
	Round,
	Trunc,
	Floor,
	Modulo,
	Min,
	Max,
	Clamp,
	Lerp,
	Lerprotate,
	Asin,
	Acos,
	Atan,
	Atan2,
	Dieroll,
	DierollInt,
	HermiteBlend,
	RandomInt,
}

// Tree Types
#[derive(Debug)]
enum Expression {
	Number(f64),
	String(String),
	Operation1(OperationType, Box<Expression>),
	Operation2(OperationType, Box<Expression>, Box<Expression>),
	Operation3(OperationType, Box<Expression>, Box<Expression>, Box<Expression>),
	Variable(String),
	QueryFunction(String),
	Allocation(String, Box<Expression>),
	ReturnStatement(Box<Expression>),
	Loop(Box<Expression>, Box<Expression>),
	Scope(Vec<Expression>)
}

fn create_operation_1(op_type: OperationType, s1: &str) -> Expression {
	Expression::Operation1(op_type, Box::new(iterate_string(s1)))
}
fn create_operation_2(op_type: OperationType, s1: &str, s2: &str) -> Expression {
	Expression::Operation2(op_type, Box::new(iterate_string(s1)), Box::new(iterate_string(s2)))
}
fn create_operation_3(op_type: OperationType, s1: &str, s2: &str, s3: &str) -> Expression {
	Expression::Operation3(op_type, Box::new(iterate_string(s1)), Box::new(iterate_string(s2)), Box::new(iterate_string(s3)))
}

fn to_variable_name(input: &str) -> String {
	if &input[1..2] == "." {
		let char = &input[0..1];
		match char {
			"q" => {return "query".to_owned() + &input[1..]},
			"v" => {return "variable".to_owned() + &input[1..]},
			"t" => {return "temp".to_owned() + &input[1..]},
			"c" => {return "context".to_owned() + &input[1..]},
			_ => ()
		}
	}
	input.to_string()
}


static STRING_NUMBER_REGEX: &str = r"^-?\d+(\.\d+f?)?$";
fn is_string_number(s: &str) -> bool {
	Regex::new(STRING_NUMBER_REGEX).unwrap().is_match(s)
}

fn can_trim_brackets(s: &str) -> bool {
	if s.starts_with('(') && s.ends_with(')') {
		let mut level: i8 = 1;
		for c in s[1..s.len()-1].chars() {
			match c {
				'(' => level += 1,
				')' => level -= 1,
				_ => {}
			}
			if level == 0 {
				return false;
			}
		}
		return true;
	}
	false
}
fn trim_brackets(input: &str) -> &str {
	if can_trim_brackets(input) {
		trim_brackets(&input[1..input.len()-1])
	} else {
		input
	}
}

fn iterate_string(input: &str) -> Expression {
	if input.len() == 0 {
		return Expression::Number(0.0);
	}
	let trimmed_input = if input.ends_with(';') {
		&input[0..input.len()-1]
	} else {
		input
	};

	let s = trim_brackets(&trimmed_input);

	if is_string_number(s) {
		let value = s.replace('f', "").parse().unwrap();
		return Expression::Number(value);
	}

	let lines = split_string_multiple(s, ";");
	if lines.len() > 1 {
		let mut expressions = Vec::new();
		for line in lines.iter() {
			let exp = iterate_string(&line);
			let is_return = matches!(exp, Expression::ReturnStatement(_));
			expressions.push(exp);
			if is_return {break;}
		}
		return Expression::Scope(expressions);
	}

	//Statement
	if s.starts_with("return") {
		return Expression::ReturnStatement(Box::new(iterate_string(&s[6..])));
	}

	match s {
		"true" => {return Expression::Number(1.0)},
		"false" => {return Expression::Number(0.0)},
		//"break" => {return Expression::Break()},
		//"continue" => {return Expression::Continue()},
		_ => {}
	}

	let has_equal_sign = s.contains('=');


	//allocation
	if has_equal_sign && s.len() > 4 {
		let mat = Regex::new(r"(temp|variable|t|v)\.\w+=").unwrap().find(s);
		match mat {
			Some(result) => {
				if &s[result.end()..result.end() + 1] != "=" {
					let name = &s[..result.end() - 1];
					let value = &s[result.end()..];
					return Expression::Allocation(to_variable_name(name), Box::new(iterate_string(&value)));
				}
			},
			None => ()
		}
	}

	// Null Coalescing
	match split_string(s, "??") {
		Some(result) => {
			return create_operation_2(OperationType::NullCoalescing, result.0, result.1);
		},
		None => ()
	}

	//ternary
	match split_string(s, "?") {
		Some(result) => {
			match split_string(result.1, ":") {
				Some(result2) => {
					return create_operation_3(OperationType::Ternary, result.0, result2.0, result2.1);
				},
				None => {
					return create_operation_2(OperationType::Ternary, result.0, result.1);
				}
			}
		},
		None => ()
	}

	//2 part operators
	match split_string(s, "&&") {
		Some(result) => { return create_operation_2(OperationType::And, result.0, result.1); },
		None => ()
	}
	match split_string(s, "||") {
		Some(result) => { return create_operation_2(OperationType::Or, result.0, result.1); },
		None => ()
	}
	if has_equal_sign {
		match split_string(s, "==") {
			Some(result) => { return create_operation_2(OperationType::Equal, result.0, result.1); },
			None => ()
		}
		match split_string(s, "!=") {
			Some(result) => { return create_operation_2(OperationType::Unequal, result.0, result.1); },
			None => ()
		}
		match split_string(s, "<=") {
			Some(result) => { return create_operation_2(OperationType::SmallerEqual, result.0, result.1); },
			None => ()
		}
	}
	match split_string(s, "<") {
		Some(result) => { return create_operation_2(OperationType::Smaller, result.0, result.1); },
		None => ()
	}
	if has_equal_sign {
		match split_string(s, ">=") {
			Some(result) => { return create_operation_2(OperationType::LargerEqual, result.0, result.1); },
			None => ()
		}
	}
	match split_string(s, ">") {
		Some(result) => { return create_operation_2(OperationType::Larger, result.0, result.1); },
		None => ()
	}

	match split_string_reverse(s, "+") {
		Some(result) => { return create_operation_2(OperationType::Add, result.0, result.1); },
		None => ()
	}
	match split_string(s, "-") {
		Some(result) => {
			if result.0.len() == 0 {
				return create_operation_1(OperationType::Invert, result.1);
			} else {
				return create_operation_2(OperationType::Subtract, result.0, result.1);
			}
		},
		None => ()
	}
	match split_string(s, "*") {
		Some(result) => { return create_operation_2(OperationType::Multiply, result.0, result.1); },
		None => ()
	}
	match split_string_reverse(s, "/") {
		Some(result) => { return create_operation_2(OperationType::Divide, result.0, result.1); },
		None => ()
	}
	if s.starts_with('!') {
		return create_operation_1(OperationType::Negate, &s[1..]);
	}

	if s.starts_with("math.") {
		if s == "math.pi" {
			return Expression::Number(std::f64::consts::PI);
		}
		let arg_begin: usize = match s.find("(") {
			Some(index) => {
				index.try_into().unwrap()
			},
			None => { 1 }
		};
		let operator = &s[5..arg_begin];
		let inner = &s[arg_begin+1..s.len()-1];

		let params = match split_string(inner, ",") {
			Some((s1, s2)) => {
				match split_string(s2, ",") {
					Some((t1, t2)) => {
						(s1, t1, t2)
					},
					None => {
						(s1, s2, "")
					}
				}
			},
			None => {
				(inner, "", "")
			}
		};

		match operator {
			"abs" => 				{return create_operation_1(OperationType::Abs, params.0)},
			"sin" => 				{return create_operation_1(OperationType::Sin, params.0)},
			"cos" => 				{return create_operation_1(OperationType::Cos, params.0)},
			"exp" => 				{return create_operation_1(OperationType::Exp, params.0)},
			"ln" => 				{return create_operation_1(OperationType::Ln, params.0)},
			"pow" => 				{return create_operation_2(OperationType::Pow, params.0, params.1)},
			"sqrt" => 				{return create_operation_1(OperationType::Sqrt, params.0)},
			"random" => 			{return create_operation_2(OperationType::Random, params.0, params.1)},
			"ceil" => 				{return create_operation_1(OperationType::Ceil, params.0)},
			"round" => 				{return create_operation_1(OperationType::Round, params.0)},
			"trunc" => 				{return create_operation_1(OperationType::Trunc, params.0)},
			"floor" => 				{return create_operation_1(OperationType::Floor, params.0)},
			"mod" => 				{return create_operation_2(OperationType::Modulo, params.0, params.1)},
			"min" => 				{return create_operation_2(OperationType::Min, params.0, params.1)},
			"max" => 				{return create_operation_2(OperationType::Max, params.0, params.1)},
			"clamp" => 				{return create_operation_3(OperationType::Clamp, params.0, params.1, params.2)},
			"lerp" => 				{return create_operation_3(OperationType::Lerp, params.0, params.1, params.2)},
			"lerprotate" => 		{return create_operation_3(OperationType::Lerprotate, params.0, params.1, params.2)},
			"asin" => 				{return create_operation_1(OperationType::Asin, params.0)},
			"acos" => 				{return create_operation_1(OperationType::Acos, params.0)},
			"atan" => 				{return create_operation_1(OperationType::Atan, params.0)},
			"atan2" => 				{return create_operation_2(OperationType::Atan2, params.0, params.1)},
			"die_roll" => 			{return create_operation_3(OperationType::Dieroll, params.0, params.1, params.2)},
			"die_roll_integer" =>	{return create_operation_3(OperationType::DierollInt, params.0, params.1, params.2)},
			"hermite_blend" => 		{return create_operation_1(OperationType::HermiteBlend, params.0)},
			"random_integer" => 	{return create_operation_2(OperationType::RandomInt, params.0, params.1)},
			_ => {return Expression::Number(0.0)}
		}
	}

	if s.starts_with("loop(") {
		let inner = &s[5..s.len()-1];
		let params = split_string_multiple(inner, ",");
		if params.len() >= 2 {
			return Expression::Loop(
				Box::new(iterate_string(params[0])),
				Box::new(iterate_string(params[1]))
			);
		}
	}

	/*split = s.match(/[a-z0-9._]{2,}/g)
	if (split && split.length === 1 && split[0].length >= s.length-2) {
		return s;
	} else if (s.includes('(') && s[s.length-1] == ')') {
		let begin = s.search(/\(/);
		let query_name = s.substr(0, begin);
		let inner = s.substr(begin+1, s.length-begin-2)
		let params = splitString(inner, ',', true);
		if (!params) params = [inner];
		
		return new QueryFunction(query_name, params);
	}*/
	return Expression::Variable(to_variable_name(&s));

	//return Expression::Number(0.0);
}




const BRACKET_OPEN: char = '(';
const BRACKET_CLOSE: char = ')';

fn split_string<'a>(s: &'a str, c: &str) -> Option<(&'a str, &'a str)> {
    if !s.contains(c) {
        return None;
    }
    let mut level: i8 = 0;
    for (i, ch) in s.char_indices() {
        if ch == BRACKET_OPEN {
            level += 1;
        } else if ch == BRACKET_CLOSE {
            level -= 1;
        } else if level == 0 && c.starts_with(ch) {
            if c.len() == 1 || &s[i..i+c.len()] == c {
                return Some((&s[..i], &s[i+c.len()..]));
            }
        }
    }
    None
}
fn split_string_reverse<'a>(s: &'a str, c: &str) -> Option<(&'a str, &'a str)> {
    if !s.contains(c) {
        return None;
    }
    let mut level: i8 = 0;
    for i in (0..s.len()).rev() {
        let ch = s.chars().nth(i).unwrap();
        if ch == BRACKET_OPEN {
            level -= 1;
        } else if ch == BRACKET_CLOSE {
            level += 1;
        } else if level == 0 && c.starts_with(ch) {
            if c.len() == 1 || &s[i..i+c.len()] == c {
                return Some((&s[..i], &s[i+c.len()..]));
            }
        }
    }
    None
}
fn split_string_multiple<'a>(s: &'a str, c: &str) -> Vec<&'a str> {
    if !s.contains(c) {
        return vec![s];
    }
	let c_len = c.len();
	let mut pieces = Vec::new();
    let mut level: i8 = 0;
	let mut last_split = 0;

    for (i, ch) in s.char_indices() {
		match ch {
			BRACKET_OPEN => {level += 1},
			BRACKET_CLOSE => {level -= 1},
			_ => {
				if level == 0 && c.starts_with(ch) {
					if c_len == 1 || &s[i..i+c_len] == c {
						let piece = &s[last_split..i];
						pieces.push(piece);
						last_split = i + c_len;
						if s[last_split..].contains(c) == false {break;}
					}
				}
			}
		}
    }
	pieces.push(&s[last_split..]);
	pieces
	
}
fn compare_values(a: &Expression, b: &Expression, variables: &mut HashMap<String, f64>) -> bool {
	let result_a = a.eval(variables);
	let result_b = b.eval(variables);
	//if (!(typeof a == 'string' && a[0] == `'`)) a = eval(a, true);
	//if (!(typeof b == 'string' && b[0] == `'`)) b = eval(b, true);
	return result_a == result_b;
}
impl Expression {
	fn eval(&self, variables: &mut HashMap<String, f64>) -> f64 {
		match self {
			Expression::Number(num) => num.to_owned(),
			Expression::String(_string) => {
				0.0
			},
			Expression::Operation1(o_type, a) => {
				let a_result = a.eval(variables);
				match o_type {
					OperationType::Negate => if a_result == 0.0 {1.0} else {0.0},
					OperationType::Invert => -a_result,
					OperationType::Abs => a_result.abs(),
					OperationType::Sin => (a_result * ANGLE_FACTOR).sin(),
					OperationType::Cos => (a_result * ANGLE_FACTOR).cos(),
					OperationType::Exp => a_result.exp(),
					OperationType::Ln => a_result.ln(),
					OperationType::Sqrt => a_result.sqrt(),
					OperationType::Ceil => a_result.ceil(),
					OperationType::Round => a_result.round(),
					OperationType::Trunc => a_result.trunc(),
					OperationType::Floor => a_result.floor(),
					OperationType::Asin => a_result.asin() * ANGLE_FACTOR,
					OperationType::Acos => a_result.acos() * ANGLE_FACTOR,
					OperationType::Atan => a_result.atan() * ANGLE_FACTOR,
					OperationType::HermiteBlend => {
						3.0 * a_result.powi(2) - 2.0 * a_result.powi(3)
					},
					_ => 0.0
				}
			},
			Expression::Operation2(o_type, a, b) => {
				let a_result = a.eval(variables);
				let b_result = b.eval(variables);
				match o_type {
					OperationType::Add => a_result + b_result,
					OperationType::Subtract => a_result - b_result,
					OperationType::Multiply => a_result * b_result,
					OperationType::Divide => a_result / b_result,
					OperationType::And => if a_result != 0.0 && b_result != 0.0 {1.0} else {0.0},
					OperationType::Or => if a_result != 0.0 || b_result != 0.0 {1.0} else {0.0},
					OperationType::Smaller => if a_result < b_result {1.0} else {0.0},
					OperationType::SmallerEqual => if a_result <= b_result {1.0} else {0.0},
					OperationType::Larger => if a_result > b_result {1.0} else {0.0},
					OperationType::LargerEqual => if a_result >= b_result {1.0} else {0.0},
					OperationType::Equal => if compare_values(a.as_ref(), b.as_ref(), variables) {1.0} else {0.0},
					OperationType::Unequal => if compare_values(a.as_ref(), b.as_ref(), variables) {0.0} else {1.0},
					OperationType::NullCoalescing => {
						// Todo
						0.0
					},
					OperationType::Pow => a_result.powf(b_result),
					OperationType::Random => {
						let mut rng = rand::thread_rng();
						rng.gen_range(a_result..b_result)
					},
					OperationType::Modulo => a_result % b_result,
					OperationType::Min => a_result.min(b_result),
					OperationType::Max => a_result.max(b_result),
					OperationType::Atan2 => a_result.atan2(b_result),
					OperationType::RandomInt => {
						let mut rng = rand::thread_rng();
						rng.gen_range(a_result..(b_result+1.0)).floor()
					},
					OperationType::Ternary => if a_result != 0.0 {b_result} else {0.0},
					_ => 0.0
				}
			},
			Expression::Operation3(o_type, a, b, c) => {
				let a_result = a.eval(variables);
				let b_result = b.eval(variables);
				let c_result = c.eval(variables);
				match o_type {
					OperationType::Clamp => a_result.clamp(b_result, c_result),
					OperationType::Lerp => a_result, //todo
					OperationType::Lerprotate => a_result, //todo
					OperationType::Dieroll => a_result, //todo
					OperationType::DierollInt => a_result, //todo
					OperationType::Ternary => if a_result != 0.0 {b_result} else {c_result},
					_ => 0.0
				}
			},
			Expression::Variable(a) => {
				
				match variables.get(a) {
					Some(value) => {
						value.to_owned()
					},
					None => {
						0.0
					}
				}
			},
			Expression::QueryFunction(a) => {
				0.0
			},
			Expression::Allocation(a, b) => {
				let value = b.eval(variables);
				variables.insert(a.clone(), value);
				0.0
			},
			Expression::ReturnStatement(a) => {
				a.eval(variables)
			},
			Expression::Loop(count, scope) => {
				let iterations = count.eval(variables) as i32;
				let mut return_value: f64 = 0.0;
				for _i in 0..iterations {
					return_value = scope.eval(variables);
				}
				return_value
			},
			Expression::Scope(lines) => {
				let mut return_value: f64 = 0.0;
				for line in lines.iter() {
					return_value = line.eval(variables);
				}
				return_value
			}
		}
	}
}

fn create_expression(string: String) -> Expression {
	
	let input = string.clone().replace(' ', "").to_lowercase();

	let expression = iterate_string(&input);
	//println!("Expression: {:?}", expression);
	expression

}

pub struct MolangParser {
	cache: HashMap<String, Expression>,
	variables: HashMap<String, f64>,
	pub enable_cache: bool
}
impl MolangParser {
	pub fn new() -> Self {
		Self {
			cache: HashMap::new(),
			variables: HashMap::new(),
			enable_cache: true
		}
	}
	pub fn parse(&mut self, input: String) -> f64 {

		if input.len() == 0 {
			return 0.0;
		}
		if input.len() < 9 && is_string_number(&input) {
			return input.parse().unwrap();
		}

		if self.enable_cache == false {
			let script = create_expression(input.to_string());
			
			return script.eval(&mut self.variables);
		}
		let cache_result = {
			self.cache.get(&input)
		};
		match cache_result {
			Some(script) => {
				script.eval(&mut self.variables)
			},
			None => {
				let script = create_expression(input.to_string());
				
				let result = script.eval(&mut self.variables);

				self.cache.insert(input.clone(), script);
				result
			}
		}
	}
}