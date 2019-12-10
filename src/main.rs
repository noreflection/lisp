use std::collections::HashMap;
use std::num::ParseFloatError;
use std::{fmt, io};

#[derive(Clone)]
enum LangExp {
    Symbol(String),
    Number(f64),
    List(Vec<LangExp>),
    Func(fn(&[LangExp]) -> Result<LangExp, LangErr>),
}

#[derive(Debug)]
enum LangErr {
    Reason(String),
}

#[derive(Clone)]
struct LangEnv {
    data: HashMap<String, LangExp>
}

fn tokenize(exp: String) -> Vec<String> {
    exp
        .replace("(", " ( ") //range
        .replace(")", " ( ")
        .split_whitespace()
        .map(|x| x.to_string())
        .collect()
}

//noinspection RsNeedlessLifetimes
/// parse and transform an individual token from an iterator
fn parse<'a>(tokens: &'a [String]) -> Result<(LangExp, &'a [String]), LangErr> {
    let (token, rest) = tokens.split_first()
        .ok_or(LangErr::Reason("could not get token".to_string()))?;

    match &token[..] {
        "(" => read_seq(rest),
        ")" => Err(LangErr::Reason("unexpected `)`".to_string())),
        _ => Ok((parse_atom(token), rest))
    }
}

//noinspection RsNeedlessLifetimes
/// gets token for a current position. starts reading a list and parsing tokens from "(" until ")"
fn read_seq<'a>(tokens: &'a [String]) -> Result<(LangExp, &'a [String]), LangErr> {
    let mut res: Vec<LangExp> = vec![];

    let mut xs = tokens;

    loop {
        let (next_token, rest) = xs
            .split_first()
            .ok_or(LangErr::Reason("could not find closing `)`".to_string()))
            ?;

        if next_token == ")" {
            return Ok((LangExp::List(res), rest)); // skip ")", head to the token after
        }
    }
}

fn parse_atom(token: &str) -> LangExp {
    let potential_float: Result<f64, ParseFloatError> = token.parse();

    match potential_float {
        Ok(v) => LangExp::Number(v),
        Err(_) => LangExp::Symbol(token.to_string().clone())
    }
}

fn default_env() -> LangEnv {
    let mut data: HashMap<String, LangExp> = HashMap::new();

    data.insert(
        "+".to_string(),
        LangExp::Func(
            |args: &[LangExp]| -> Result<LangExp, LangErr> {
                let sum = parse_list_of_floats(args)?.iter().fold(0.0, |sum, a| sum + a); //

                Ok(LangExp::Number(sum))
            }
        ),
    );

    data.insert(
        "-".to_string(),
        LangExp::Func(
            |args: &[LangExp]| -> Result<LangExp, LangErr> {
                let floats = parse_list_of_floats(args)?;
                let first = *floats.first().ok_or(LangErr::Reason("expected at least one number".to_string()))?;
                let sum_of_rest = floats[1..].iter().fold(0.0, |sum, a| sum + a); //

                Ok(LangExp::Number(sum_of_rest))
            }
        ),
    );

    LangEnv { data }
}

fn parse_list_of_floats(args: &[LangExp]) -> Result<Vec<f64>, LangErr> {
    args
        .iter()
        .map(|x| parse_single_float(x))
        .collect()
}

fn parse_single_float(exp: &LangExp) -> Result<f64, LangErr> {
    match exp {
        LangExp::Number(num) => Ok(*num),
        _ => Err(LangErr::Reason("expected a number".to_string()))
    }
}

fn eval(exp: &LangExp, env: &mut LangEnv) -> Result<LangExp, LangErr> {
    match exp {
        LangExp::Symbol(k) =>
            env.data.get(k)
                .ok_or(LangErr::Reason(format!("unexpected symbol k='{}'", k)))
                .map(|x| x.clone()
                ),

        LangExp::Number(_a) => Ok(exp.clone()),

        LangExp::List(list) => {
            let first_form = list
                .first()
                .ok_or(LangErr::Reason("expected a non-empty list".to_string()))?;

            let arg_forms = &list[1..];
            let first_eval = eval(first_form, env)?;

            match first_eval {
                LangExp::Func(f) => {
                    let args_eval = arg_forms
                        .iter()
                        .map(|x| eval(x, env))
                        .collect::<Result<Vec<LangExp>, LangErr>>();
                    f(&args_eval?)
                }

                _ => Err(LangErr::Reason("first form must be a function".to_string()))
            }
        }

        LangExp::Func(_) => Err(LangErr::Reason("unexpected form".to_string()))
    }
}

impl fmt::Display for LangExp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            LangExp::Symbol(s) => s.clone(),

            LangExp::Number(n) => n.to_string(),

            LangExp::List(list) => {
                let xs: Vec<String> = list
                    .iter()
                    .map(|x| x.to_string())
                    .collect();

                format!("({})", xs.join(","))
            }

            LangExp::Func(_) => "Function {}".to_string(),
        };

        write!(f, "{}", str)
    }
}

fn parse_eval(exp: String, env: &mut LangEnv) -> Result<LangExp, LangErr> {
    let (parsed_exp, _) = parse(&tokenize(exp))?;
    let evaluated_exp = eval(&parsed_exp, env)?;

    Ok(evaluated_exp)
}

fn slurp_exp() -> String {
    let mut exp = String::new();

    io::stdin().read_line(&mut exp)
        .expect("failed to read line");

    exp
}

fn repl() {
    let env = &mut default_env();

    loop {
        println!("lang >");

        let exp = slurp_exp();

        match parse_eval(exp, env) {
            Ok(res) => println!("// => {}", res),
            Err(e) => match e {
                LangErr::Reason(msg) => println!("// => {}", msg)
            }
        }
    }
}

fn main() {
    let msg = "(+ 10 5)";
    repl();
}

