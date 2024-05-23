use std::collections::HashMap;

use crate::ir_gen::ast::*;

pub trait Calc {
    fn calculate(&self, vars_table: &mut HashMap<String, Variable>) -> i32;
}

impl Calc for ConstInitVal {
    fn calculate(&self, vars_table: &mut HashMap<String, Variable>) -> i32 {
        self.const_exp.calculate(vars_table)
    }
}

impl Calc for ConstExp {
    fn calculate(&self, vars_table: &mut HashMap<String, Variable>) -> i32 {
        self.exp.calculate(vars_table)
    }
}

impl Calc for Exp {
    fn calculate(&self, vars_table: &mut HashMap<String, Variable>) -> i32 {
        self.lor_exp.calculate(vars_table)
    }
}

impl Calc for LOrExp {
    fn calculate(&self, vars_table: &mut HashMap<String, Variable>) -> i32 {
        match self {
            LOrExp::LAndExp(land_exp) => land_exp.calculate(vars_table),
            LOrExp::LOrExp((lor_exp, land_exp)) => {
                (lor_exp.calculate(vars_table) != 0 || land_exp.calculate(vars_table) != 0) as i32
            }
        }
    }
}

impl Calc for LAndExp {
    fn calculate(&self, vars_table: &mut HashMap<String, Variable>) -> i32 {
        match self {
            LAndExp::EqExp(eq_exp) => eq_exp.calculate(vars_table),
            LAndExp::LAndExp((land_exp, eq_exp)) => {
                (land_exp.calculate(vars_table) != 0 && eq_exp.calculate(vars_table) != 0) as i32
            }
        }
    }
}

impl Calc for EqExp {
    fn calculate(&self, vars_table: &mut HashMap<String, Variable>) -> i32 {
        match self {
            EqExp::RelExp(rel_exp) => rel_exp.calculate(vars_table),
            EqExp::EqExp((eq_exp, cmp_op, rel_exp)) => match cmp_op {
                CmpOp::Eq => (eq_exp.calculate(vars_table) == rel_exp.calculate(vars_table)) as i32,
                CmpOp::NEq => {
                    (eq_exp.calculate(vars_table) != rel_exp.calculate(vars_table)) as i32
                }
                _ => unreachable!(),
            },
        }
    }
}

impl Calc for RelExp {
    fn calculate(&self, vars_table: &mut HashMap<String, Variable>) -> i32 {
        match self {
            RelExp::AddExp(add_exp) => add_exp.calculate(vars_table),
            RelExp::CompExp((rel_exp, cmp_op, add_exp)) => match cmp_op {
                CmpOp::Less => {
                    (rel_exp.calculate(vars_table) < add_exp.calculate(vars_table)) as i32
                }
                CmpOp::Grate => {
                    (rel_exp.calculate(vars_table) > add_exp.calculate(vars_table)) as i32
                }
                CmpOp::LessEq => {
                    (rel_exp.calculate(vars_table) <= add_exp.calculate(vars_table)) as i32
                }
                CmpOp::GrateEq => {
                    (rel_exp.calculate(vars_table) >= add_exp.calculate(vars_table)) as i32
                }
                _ => unreachable!(),
            },
        }
    }
}

impl Calc for AddExp {
    fn calculate(&self, vars_table: &mut HashMap<String, Variable>) -> i32 {
        match self {
            AddExp::MulExp(mul_exp) => mul_exp.calculate(vars_table),
            AddExp::AddExp((add_exp, add_op, mul_exp)) => match add_op {
                AddOp::Add => add_exp.calculate(vars_table) + mul_exp.calculate(vars_table),
                AddOp::Sub => add_exp.calculate(vars_table) - mul_exp.calculate(vars_table),
            },
        }
    }
}

impl Calc for MulExp {
    fn calculate(&self, vars_table: &mut HashMap<String, Variable>) -> i32 {
        match self {
            MulExp::UnaryExp(unary_exp) => unary_exp.calculate(vars_table),
            MulExp::MulExp((mul_exp, mul_op, unary_exp)) => match mul_op {
                MulOp::Multiple => mul_exp.calculate(vars_table) * unary_exp.calculate(vars_table),
                MulOp::Divide => mul_exp.calculate(vars_table) / unary_exp.calculate(vars_table),
                MulOp::Mod => mul_exp.calculate(vars_table) % unary_exp.calculate(vars_table),
            },
        }
    }
}

impl Calc for UnaryExp {
    fn calculate(&self, vars_table: &mut HashMap<String, Variable>) -> i32 {
        match self {
            UnaryExp::PrimaryExp(pri_exp) => pri_exp.calculate(vars_table),
            UnaryExp::UnaryExp((unary_op, unary_exp)) => match unary_op {
                UnaryOp::Passive => unary_exp.calculate(vars_table),
                UnaryOp::Negative => -unary_exp.calculate(vars_table),
                UnaryOp::Inversion => (unary_exp.calculate(vars_table) == 0) as i32,
            },
            _ => unreachable!(),
        }
    }
}

impl Calc for PrimaryExp {
    fn calculate(&self, vars_table: &mut HashMap<String, Variable>) -> i32 {
        match self {
            PrimaryExp::Exp(exp) => exp.calculate(vars_table),
            PrimaryExp::Number(int) => *int,
            PrimaryExp::LVal(var) => match vars_table.get(var) {
                Some(int) => match int {
                    Variable::ConstINT(const_int) => *const_int,
                    _ => unreachable!(),
                },
                None => unreachable!(),
            },
        }
    }
}
