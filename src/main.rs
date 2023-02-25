use crate::molang::MolangParser;
use std::time::Instant;

#[macro_use]
extern crate lazy_static;

pub mod molang;


fn test_performance() {
	let mut parser = MolangParser::new();
	//parser.enable_cache = false;

	let start = Instant::now();

	for _i in 0..100_000 {
		let input = "false ? 5 : (20 * math.pow(2+2, 2))".to_string();
		parser.parse(input);
	}

	let duration = start.elapsed();

	println!("Output: in {:?}", duration);
}


fn main() {

	test_performance();

}

#[cfg(test)]
mod tests {
	fn run(input: &str) -> f32 {
		use crate::molang::MolangParser;
		let mut parser = MolangParser::new();

		parser.parse(input.to_string())
	}
	#[test]
	fn basic() {
		assert_eq!(run("1+1"), 2.0);
	}
	#[test]
	fn sign() {
		assert_eq!(run("2*-(2/2)"), -2.0);
	}
	#[test]
	fn order_of_operation() {
		assert_eq!(run("1 + 1 * 2"), 3.0);
	}
	#[test]
	fn order_of_operation_2() {
		assert_eq!(run("18 - 2 * -0.5"), 19.0);
	}
	#[test]
	fn float_type_notation() {
		assert_eq!(run("10 * 0.2f",), 2.0);
	}
	#[test]
	fn order_of_division() {
		assert_eq!(run("12 / 2 / 2"), 3.0);
	}
	#[test]
	fn binary() {
		assert_eq!(run("true ? 10"), 10.0);
	}
	#[test]
	fn ternary() {
		assert_eq!(run("false ? 5 : 10"), 10.0);
	}
	#[test]
	fn greater_or_equal() {
		assert_eq!(run("3 >= 4"), 0.0);
	}
	#[test]
	fn multi_line() {
		assert_eq!(run("temp.test = 33; return temp.test * 2"), 66.0);
	}
	#[test]
	fn return_value() {
		assert_eq!(run("temp.test = 4; return temp.test; return 5;"), 4.0);
	}
	#[test]
	fn math() {
		assert_eq!(run("Math.pow(Math.clamp(500, 0, 3), 2)"), 9.0);
	}
	#[test]
	fn aliases() {
		assert_eq!(run("t.a = 6; variable.b = 2; return temp.a / v.b;"), 3.0);
	}
	#[test]
	fn lerprotate() {
		assert_eq!(run("Math.lerprotate(10, 380, 0.5) + Math.lerprotate(50, -10, 0.25)"), 20.0);
	}
	#[test]
	fn inverse_trigonometry() {
		assert_eq!(run("Math.round(Math.acos(-1) + Math.atan2(2, 4))"), 207.0);
	}
	#[test]
	fn query_in_range() {
		assert_eq!(run("q.in_range(1, 0, 2) && !query.in_range(55, 1, 5)"), 1.0);
	}
	#[test]
	fn query_approx_eq() {
		assert_eq!(run("q.approx_eq(2, 2.00000000002) && !q.approx_eq(2, 2, 3)"), 1.0);
	}
	#[test]
	fn loops() {
		assert_eq!(run("v.count = 0; loop(10, {v.count = v.count + 1}); return v.count;"), 10.0);
	}
	#[test]
	fn not_enough_arguments() {
		assert_eq!(run("Math.pow()"), 1.0);
	}
	#[test]
	fn broken_expression() {
		assert_eq!(run(")22 + 5 * (v.something"), 0.0);
	}
}
