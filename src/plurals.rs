use self::Resolver::*;

#[derive(Clone, Debug)]
pub enum Resolver {
    /// A boolean expression
    /// Use Ast::parse to get an Ast
    Expr(Box<Ast>),
}


fn index_of<'a>(src: &'a str, pat: &'static str) -> Option<usize> {
   src.chars().fold((None, 0, 0, 0), |(match_index, i, n_matches, paren_level), ch| {
       if let Some(x) = match_index {
           return (Some(x), i, n_matches, paren_level);
       } else {
           let new_par_lvl = match ch {
               '(' => paren_level + 1,
               ')' => paren_level - 1,
                _ => paren_level
           };

           if Some(ch) == pat.chars().nth(n_matches) {
               let length = n_matches + 1;
               if length == pat.len() && new_par_lvl == 0 {
                   (Some(i - n_matches), i + 1, length, new_par_lvl)
               } else {
                   (match_index, i + 1, length, new_par_lvl)
               }
           } else {
               (match_index, i + 1, 0, new_par_lvl)
           }
       }
    }).0
}

use self::Ast::*;
#[derive(Clone, Debug, PartialEq)]
pub enum Ast {
    /// A ternary expression
    /// x ? a : b
    ///
    /// the three Ast<'a> are respectively x, a and b.
    Ternary(Box<Ast>, Box<Ast>, Box<Ast>),
    /// The n variable.
    N,
    /// Integer literals.
    Integer(u64),
    /// Binary operators.
    Op(Operator, Box<Ast>, Box<Ast>),
    /// ! operator.
    Not(Box<Ast>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Equal,
    NotEqual,
    GreaterOrEqual,
    SmallerOrEqual,
    Greater,
    Smaller,
    And,
    Or,
    Modulo,
}

impl Ast {
    fn resolve(&self, n: u64) -> usize {
        match *self {
            Ternary(ref cond, ref ok, ref nok) => if cond.resolve(n) == 0 {
                nok.resolve(n)
            } else {
                ok.resolve(n)
            },
            N => n as usize,
            Integer(x) => x as usize,
            Op(ref op, ref lhs, ref rhs) => match *op {
                Operator::Equal => (lhs.resolve(n) == rhs.resolve(n)) as usize,
                Operator::NotEqual => (lhs.resolve(n) != rhs.resolve(n)) as usize,
                Operator::GreaterOrEqual => (lhs.resolve(n) >= rhs.resolve(n)) as usize,
                Operator::SmallerOrEqual => (lhs.resolve(n) <= rhs.resolve(n)) as usize,
                Operator::Greater => (lhs.resolve(n) > rhs.resolve(n)) as usize,
                Operator::Smaller => (lhs.resolve(n) < rhs.resolve(n)) as usize,
                Operator::And => (lhs.resolve(n) != 0 && rhs.resolve(n) != 0) as usize,
                Operator::Or => (lhs.resolve(n) != 0 || rhs.resolve(n) != 0) as usize,
                Operator::Modulo => lhs.resolve(n) % rhs.resolve(n),
            },
            Not(ref val) => match val.resolve(n) {
                0 => 1,
                _ => 0,
            }
        }
    }

    pub fn parse<'a>(src: &'a str) -> Ast {
        Self::parse_parens(src.trim())
    }

    fn parse_parens<'a>(src: &'a str) -> Ast {
        if src.starts_with('(') && src.ends_with(')') {
            Ast::parse(src[1..src.len() - 1].trim())
        } else {
            Ast::parse_and(src.trim())
        }
    }

    fn parse_and<'a>(src: &'a str) -> Ast {
        if let Some(i) = index_of(src, "&&") {
            Ast::Op(Operator::And, Box::new(Ast::parse(&src[0..i])), Box::new(Ast::parse(&src[i + 2..])))
        } else {
            Self::parse_or(src)
        }
    }

    fn parse_or<'a>(src: &'a str) -> Ast {
        if let Some(i) = index_of(src, "||") {
            Ast::Op(Operator::Or, Box::new(Ast::parse(&src[0..i])), Box::new(Ast::parse(&src[i + 2..])))
        } else {
            Self::parse_ternary(src)
        }
    }

    fn parse_ternary<'a>(src: &'a str) -> Ast {
        if let Some(i) = index_of(src, "?") {
            if let Some(l) = index_of(src, ":") {
                Ast::Ternary(
                    Box::new(Ast::parse(&src[0..i])),
                    Box::new(Ast::parse(&src[i + 1..l])),
                    Box::new(Ast::parse(&src[l + 1..]))
                )
            } else {
                panic!("Incorrect ternary expression, expected `:`")
            }
        } else {
            Self::parse_ge(src)
        }
    }

    fn parse_ge<'a>(src: &'a str) -> Ast {
        if let Some(i) = index_of(src, ">=") {
            Ast::Op(Operator::GreaterOrEqual, Box::new(Ast::parse(&src[0..i])), Box::new(Ast::parse(&src[i + 2..])))
        } else {
            Self::parse_gt(src)
        }
    }

    fn parse_gt<'a>(src: &'a str) -> Ast {
        if let Some(i) = index_of(src, ">") {
            Ast::Op(Operator::Greater, Box::new(Ast::parse(&src[0..i])), Box::new(Ast::parse(&src[i + 1..])))
        } else {
            Self::parse_le(src)
        }
    }

    fn parse_le<'a>(src: &'a str) -> Ast {
        if let Some(i) = index_of(src, "<=") {
            Ast::Op(Operator::SmallerOrEqual, Box::new(Ast::parse(&src[0..i])), Box::new(Ast::parse(&src[i + 2..])))
        } else {
            Self::parse_lt(src)
        }
    }

    fn parse_lt<'a>(src: &'a str) -> Ast {
        if let Some(i) = index_of(src, "<") {
            Ast::Op(Operator::Smaller, Box::new(Ast::parse(&src[0..i])), Box::new(Ast::parse(&src[i + 1..])))
        } else {
            Self::parse_eq(src)
        }
    }

    fn parse_eq<'a>(src: &'a str) -> Ast {
        if let Some(i) = index_of(src, "==") {
            Ast::Op(Operator::Equal, Box::new(Ast::parse(&src[0..i])), Box::new(Ast::parse(&src[i + 2..])))
        } else {
            Self::parse_neq(src)
        }
    }

    fn parse_neq<'a>(src: &'a str) -> Ast {
        if let Some(i) = index_of(src, "!=") {
            Ast::Op(Operator::NotEqual, Box::new(Ast::parse(&src[0..i])), Box::new(Ast::parse(&src[i + 2..])))
        } else {
            Self::parse_mod(src)
        }
    }
    fn parse_mod<'a>(src: &'a str) -> Ast {
        if let Some(i) = index_of(src, "%") {
            Ast::Op(Operator::Modulo, Box::new(Ast::parse(&src[0..i])), Box::new(Ast::parse(&src[i + 1..])))
        } else {
            Self::parse_not(src.trim())
        }
    }

    fn parse_not<'a>(src: &'a str) -> Ast {
        if index_of(src, "!") == Some(0) {
            Ast::Not(Box::new(Ast::parse(&src[1..])))
        } else {
            Self::parse_int(src.trim())
        }
    }

    fn parse_int<'a>(src: &'a str) -> Ast {
        u64::from_str_radix(src, 10)
            .map(|x| Ast::Integer(x))
            .unwrap_or_else(|_| Self::parse_n(src.trim()))
    }

    fn parse_n<'a>(src: &'a str) -> Ast {
        if src == "n" {
            Ast::N
        } else {
            panic!("Unexpected token: {}", src)
        }
    }
}

impl Resolver {
    /// Returns the number of the correct plural form
    /// for `n` objects, as defined by the rule contained in this resolver.
    pub fn resolve(&self, n: u64) -> usize {
        match *self {
            Expr(ref ast) => {
                ast.resolve(n)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr_resolver() {
        assert_eq!(Expr(Box::new(N)).resolve(42), 42);
    }

    #[test]
    fn test_parser() {
        assert_eq!(Ast::parse("n == 42 ? n : 6 && n < 7"), Ast::Op(
            Operator::And,
            Box::new(Ast::Ternary(
                Box::new(Ast::Op(
                    Operator::Equal,
                    Box::new(Ast::N),
                    Box::new(Ast::Integer(42))
                )),
                Box::new(Ast::N),
                Box::new(Ast::Integer(6))
            )),
            Box::new(Ast::Op(
                Operator::Smaller,
                Box::new(Ast::N),
                Box::new(Ast::Integer(7))
            ))
        ));

        assert_eq!(Ast::parse("(n)"), Ast::N);

        assert_eq!(Ast::parse("(n == 1 || n == 2) ? 0 : 1"), Ast::Ternary(
            Box::new(Ast::Op(
                Operator::Or,
                Box::new(Ast::Op(
                    Operator::Equal,
                    Box::new(Ast::N),
                    Box::new(Ast::Integer(1))
                )),
                Box::new(Ast::Op(
                    Operator::Equal,
                    Box::new(Ast::N),
                    Box::new(Ast::Integer(2))
                ))
            )),
            Box::new(Ast::Integer(0)),
            Box::new(Ast::Integer(1))
        ))
    }
}
