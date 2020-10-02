use std::fs::File;
use std::io::prelude::*;
use std::io::Result;

use dsl::*;
use dsl::Constraint::*;

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
    match &prob.obj_expr_arena {
        Some(expr_arena) => {
            format!("{}obj: {}", obj_type, expr_arena.to_lp_file_format())
        }
        _ => String::new()
    }
}
fn constraints_lp_file_block(prob: &LpProblem) -> String {
    let mut res = String::new();
    let mut constraints = prob.constraints.iter();
    let mut index = 1;
    while let Some(constraint) = constraints.next() {
        res.push_str(&format!("  c{}: {}\n", index.to_string(), constraint.to_lp_file_format()));
        index += 1;
    }
    res
}

fn bounds_lp_file_block(prob: &LpProblem) -> String {
    let mut res = String::new();
    for (_, (constraint_index, lp_expr_arena_index)) in prob.variables() {
        let expr_ref = prob.constraints.get(constraint_index).unwrap().0.expr_ref_at(lp_expr_arena_index);
        match expr_ref {
            &LpExpression::ConsInt(LpInteger {
                         ref name,
                         lower_bound,
                         upper_bound,
                     })
            | &LpExpression::ConsCont(LpContinuous {
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
                    match expr_ref {
                        &LpExpression::ConsCont(LpContinuous { .. }) => {
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
    for (_, (constraint_index, lp_expr_arena_index)) in prob.variables() {
        match prob.constraints.get(constraint_index).unwrap().0.expr_ref_at(lp_expr_arena_index) {
            &LpExpression::ConsInt(LpInteger { ref name, .. }) => {
                res.push_str(format!("{} ", name).as_str());
            }
            _ => (),
        }
    }
    res
}

fn binaries_lp_file_block(prob: &LpProblem) -> String  {
    let mut res = String::new();
    for (_, (constraint_index, lp_expr_arena_index)) in prob.variables() {
        match prob.constraints.get(constraint_index).unwrap().0.expr_ref_at(lp_expr_arena_index) {
            &LpExpression::ConsBin(LpBinary { ref name }) => {
                res.push_str(format!("{} ", name).as_str());
            }
            _ => (),
        }
    }
    res
}

impl LpFileFormat for LpExprArena {
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
        let root_index = self.get_root_index();
        let mut clone = self.clone();
        formalize_signs(clone.simplify().show(&root_index, false))
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
