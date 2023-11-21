//! Responsible for doing static analysis operation on the code before AST creation

mod parsers;

use nom::{
    branch::*, bytes::complete::tag, character, combinator::opt, multi::*, sequence::tuple, Parser,
};
use parsers::*;

use crate::parsers::{types::Position, *};

/// Contains useful data about the code
#[derive(Debug, Clone)]
pub struct Analysis<'a> {
    /// Functions that's hoisted
    /// - This is only possible for functions that has a function assigned to it
    pub hoisted_funcs: Vec<FunctionInfo<'a>>,
}

#[derive(Debug, Clone)]
/// Information for a function
/// # Note
/// - This only applies for functions defined with `function` keyword and functions assigned to a variable
pub struct FunctionInfo<'a> {
    pub identifier: &'a str,
    pub arg_count: usize,
    /// Index of the line where the function will become usable
    pub hoisted_line: usize,
    /// Where the expression / scope is located as an index
    pub body_location: usize,
}

impl<'a> Analysis<'a> {
    /// Does a static analysis of code
    pub fn analyze(input: &'a str) -> Self {
        let comma = || character::complete::char(',');
        let arg = identifier(tuple((comma(), ws)));
        let args = separated_list0(comma(), arg);
        let arrow = tag("=>");
        let func_expression =
            tuple((opt(args), ws1, arrow, till_term)).map(|(args, _, arrow, _)| (args, arrow));
        let var_decl_func = var_decl(func_expression).map(
            |(var_decl_pos, identifier, life_time, (args, expr_pos))| {
                FunctionInfo {
                    identifier: identifier.input,
                    arg_count: match args {
                        Some(args) => args.len(),
                        None => 0,
                    },
                    hoisted_line: match life_time {
                        Some(life_time) => match life_time {
                            LifeTime::Infinity => var_decl_pos.line, // positive infinity
                            LifeTime::Seconds(_) => var_decl_pos.line,
                            LifeTime::Lines(lines) => {
                                var_decl_pos.line.saturating_add_signed(lines)
                            }
                        },
                        None => var_decl_pos.line,
                    },
                    body_location: expr_pos.index,
                }
            },
        );

        // TODO comment
        let input = Position::new(input);
        let (_, hoisted_funcs) = fold_many0(
            alt((ws.map(|_| None), var_decl_func.map(Some))),
            Vec::new,
            |mut vec, item| {
                if let Some(item) = item {
                    vec.push(item)
                }
                vec
            },
        )(input)
        .unwrap();

        Self { hoisted_funcs }
    }
}

fn is_valid_lifetime(mut life_time: &str) -> bool {
    if life_time == "Infinity" {
        return true;
    }

    // check if seconds
    if life_time.ends_with('s') {
        if life_time.starts_with('-') {
            // negative seconds...
            return false;
        }

        life_time = &life_time[..life_time.len() - 1];
        return life_time.parse::<f64>().is_ok();
    }

    // lines, which can be negative so just pass as number
    life_time.parse::<f64>().is_ok()
}
