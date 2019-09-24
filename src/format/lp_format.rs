use std::fs::File;
use std::io::prelude::*;
use std::io::Result;

use dsl::*;
use dsl::Constraint::*;
use dsl::LpExpression::*;

pub trait LpFileFormat {
    fn to_lp_file_format(&self) -> String;
    fn write_lp(&self, file_model: &str) -> Result<()> {
        let mut buffer = File::create(file_model)?;
        buffer.write(self.to_lp_file_format().as_bytes())?;
        Ok(())
    }
}

impl LpFileFormat for LpProblem {

    fn to_lp_file_format(&self) -> String {

        let mut buffer = String::new();

        buffer.push_str(format!("\\ {}\n\n", &self.name).as_str());

        buffer.push_str( &objective_lp_file_block(self) );

        let constraints_block = constraints_lp_file_block(self);
        if constraints_block.len() > 0 {
            buffer.push_str(format!("\n\nSubject To\n{}", &constraints_block).as_str());
        }

        let bounds_block = bounds_lp_file_block(self);
        if bounds_block.len() > 0 {
            buffer.push_str(format!("\nBounds\n{}", &bounds_block).as_str());
        }

        let integers_block = integers_lp_file_block(self);
        if integers_block.len() > 0 {
            buffer.push_str(format!("\nGenerals\n  {}\n", &integers_block).as_str());
        }

        let binaries_block = binaries_lp_file_block(self);
        if binaries_block.len() > 0 {
            buffer.push_str(format!("\nBinary\n  {}\n", &binaries_block).as_str());
        }

        buffer.push_str("\nEnd");

        buffer
    }
}

fn objective_lp_file_block(prob: &LpProblem) -> String {
    // Write objectives
    let obj_type = match prob.objective_type {
        LpObjective::Maximize => "Maximize\n  ",
        LpObjective::Minimize => "Minimize\n  "
    };
    match prob.obj_expr {
        Some(ref expr) => format!("{}obj: {}", obj_type, expr.to_lp_file_format()),
        _ => String::new()
    }
}
fn constraints_lp_file_block(prob: &LpProblem) -> String {
    let mut res = String::new();
    let mut constraints = prob.constraints.iter();
    let mut index = 1;
    while let Some(ref constraint) = constraints.next() {
        res.push_str(&format!("  c{}: {}\n", index.to_string(), &constraint.to_lp_file_format()));
        index += 1;
    }
    res
}

fn bounds_lp_file_block(prob: &LpProblem) -> String {
    let mut res = String::new();
    for (_, v) in prob.variables() {
        match v {
            &ConsInt(LpInteger {
                         ref name,
                         lower_bound,
                         upper_bound,
                     })
            | &ConsCont(LpContinuous {
                            ref name,
                            lower_bound,
                            upper_bound,
                        }) => {
                if let Some(l) = lower_bound {
                    res.push_str(&format!("  {} <= {}", &l.to_string(), &name));
                    if let Some(u) = upper_bound {
                        res.push_str(&format!(" <= {}", &u.to_string()));
                    }
                    res.push_str("\n");
                } else if let Some(u) = upper_bound {
                    res.push_str(&format!("  {} <= {}\n", &name, &u.to_string()));
                } else {
                    match v {
                        &ConsCont(LpContinuous { .. }) => {
                            res.push_str(&format!("  {} free\n", &name));
                        } // TODO: IntegerVar => -INF to INF
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
    res
}

fn integers_lp_file_block(prob: &LpProblem) -> String {
    let mut res = String::new();
    for (_, v) in prob.variables() {
        match v {
            &ConsInt(LpInteger { ref name, .. }) => {
                res.push_str(format!("{} ", name).as_str());
            }
            _ => (),
        }
    }
    res
}

fn binaries_lp_file_block(prob: &LpProblem) -> String  {
    let mut res = String::new();
    for (_, v) in prob.variables() {
        match v {
            &ConsBin(LpBinary { ref name }) => {
                res.push_str(format!("{} ", name).as_str());
            }
            _ => (),
        }
    }
    res
}

impl LpFileFormat for LpExpression {
    fn to_lp_file_format(&self) -> String {
        fn formalize_signs(s: String) -> String {
            let mut s = s.clone();
            let mut t = "".to_string();
            while s != t {
                t = s.clone();
                s = s.replace("+ +", "+ ");
                s = s.replace("- +", "- ");
                s = s.replace("+ -", "- ");
                s = s.replace("- -", "+ ");
                s = s.replace("  ", " ");
            }
            s
        }

        formalize_signs(show(&simplify(self), false))
    }
}

fn show(e: &LpExpression, with_parenthesis: bool) -> String {
    let str_left_mult = if with_parenthesis { "(" } else { "" };
    let str_right_mult = if with_parenthesis { ")" } else { "" };
    let str_op_mult = if with_parenthesis { " * " } else { " " };
    match e {
        &LitVal(n) => n.to_string(),
        &AddExpr(ref e1, ref e2) => {
            str_left_mult.to_string()
                + &show(e1, with_parenthesis)
                + " + "
                + &show(e2, with_parenthesis)
                + str_right_mult
        }
        &SubExpr(ref e1, ref e2) => {
            str_left_mult.to_string()
                + &show(e1, with_parenthesis)
                + " - "
                + &show(e2, with_parenthesis)
                + str_right_mult
        }
        &MulExpr(ref e1, ref e2) => {
            let ref deref_e1 = **e1;

            match deref_e1 {
                &LitVal(v) if v == 1.0 => {
                    //e2.to_lp_file_format()
                    str_left_mult.to_string()
                        + &" ".to_string()
                        + &show(e2, with_parenthesis)
                        + str_right_mult
                }
                &LitVal(v) if v == -1.0 => {
                    //"-".to_string() + &e2.to_lp_file_format()
                    str_left_mult.to_string()
                        + &"-".to_string()
                        + &show(e2, with_parenthesis)
                        + str_right_mult
                }
                _ => {
                    str_left_mult.to_string()
                        + &show(e1, with_parenthesis)
                        + str_op_mult
                        + &show(e2, with_parenthesis)
                        + str_right_mult
                }
            }
        }
        &ConsBin(LpBinary { name: ref n, .. }) => n.to_string(),
        &ConsInt(LpInteger { name: ref n, .. }) => n.to_string(),
        &ConsCont(LpContinuous { name: ref n, .. }) => n.to_string(),
        _ => "EmptyExpr!!".to_string(),
    }
}

impl LpFileFormat for LpConstraint {
    fn to_lp_file_format(&self) -> String {
        let mut res = String::new();
        res.push_str(&self.0.to_lp_file_format());
        match self.1 {
            GreaterOrEqual => res.push_str(" >= "),
            LessOrEqual => res.push_str(" <= "),
            Equal => res.push_str(" = "),
        }
        res.push_str(&self.2.to_lp_file_format());
        res
    }
}
