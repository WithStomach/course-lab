use crate::ir_gen::ast::*;
use std::{collections::HashMap, fmt::format};

use super::calc::Calc;

impl CompUnit {
    /// generate koopa ir from a CompUnit in String form
    pub fn generate_koopa(&self) -> String {
        let mut compiler_info = CompilerInfo {
            temp_id: 0,
            vars_table: HashMap::new(),
        };
        self.show(&mut compiler_info).0
    }
}

#[derive(Debug, PartialEq, Clone)]
struct CompilerInfo {
    pub temp_id: i32,
    pub vars_table: HashMap<String, Variable>,
}

enum Res {
    Nothing,
    Imm,
    Temp(i32),
}

trait Show {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res);
}

impl Show for CompUnit {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        self.func_def.show(info)
    }
}

impl Show for FuncDef {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "fun @".to_string();
        s += &self.id;
        s += "(): ";
        match self.func_type {
            ItemType::Int => {
                s += "i32";
            }
            ItemType::Double => {
                s += "double";
            }
        }
        s += "{\n";
        s += &self.block.show(info).0;
        s += "}\n";
        (s, Res::Nothing)
    }
}

impl Show for Block {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "%entry:\n".to_string();
        let mut block_info = info.clone();
        for item in &self.items {
            s += &item.show(&mut block_info).0;
        }
        (s, Res::Nothing)
    }
}

impl Show for BlockItem {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        match self {
            BlockItem::Decl(decl) => {
                s += &*decl.show(info).0;
            }
            BlockItem::Stmt(stmt) => {
                s += &*stmt.show(info).0;
            }
        }
        (s, Res::Nothing)
    }
}

impl Show for Decl {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        match self {
            Decl::ConstDecl(const_decl) => const_decl.show(info),
            Decl::VarDecl(var_decl) => var_decl.show(info),
        }
    }
}

impl Show for VarDecl {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        match self.b_type {
            ItemType::Int => {}
            _ => unreachable!(),
        }
        for var_def in &self.var_defs {
            s += &var_def.show(info).0;
        }
        (s, Res::Nothing)
    }
}

impl Show for ConstDecl {
    /// 目前只能处理int类型的常量定义
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        match self.b_type {
            ItemType::Int => {}
            _ => unreachable!(),
        }
        for const_def in &self.const_defs {
            const_def.show(info);
        }
        ("".to_string(), Res::Nothing)
    }
}

impl Show for ConstDef {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        // 在变量表中寻找是否该变量已经被定义
        match info.vars_table.get(&self.ident) {
            // 若未被定义，将其添加进变量表中
            None => {
                let value = self.const_init_val.calculate(&mut info.vars_table);
                info.vars_table
                    .insert(self.ident.clone(), Variable::ConstINT(value));
            }
            // 否则，重复定义常量，报错
            Some(value) => unreachable!(),
        }
        ("".to_string(), Res::Nothing)
    }
}

impl Show for VarDef {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        match self {
            VarDef::Def((ident, init_val)) => {
                // 首先检查该变量是否已被定义。
                match info.vars_table.get(ident) {
                    // 若已被定义，属于重复定义，报错
                    Some(int) => unreachable!(),
                    // 若尚未被定义，将其加入变量表
                    None => {
                        // 生成该变量对应的指针的名字：@ident
                        let var = Variable::INT(format!("@{0}", ident));
                        // 将其插入变量表中
                        info.vars_table.insert(ident.clone(), var);
                        // 为该变量进行alloc操作
                        s += &format!("    @{0} = alloc i32\n", ident);
                        // 计算init_val
                        let (init_s, init_res) = init_val.show(info);
                        // 将结果存进内存
                        match init_res {
                            Res::Nothing => unreachable!(),
                            Res::Imm => {
                                s += &format!("    store {0}, @{1}\n", init_s, ident);
                            }
                            Res::Temp(idx) => {
                                s += &init_s;
                                s += &format!("    store %{0}, @{1}\n", idx, ident);
                            }
                        }
                    }
                }
            }
            VarDef::Decl(ident) => {
                match info.vars_table.get(ident) {
                    // 若已被定义，属于重复定义，报错
                    Some(int) => unreachable!(),
                    // 若尚未被定义，将其加入变量表
                    None => {
                        // 生成该变量对应的指针的名字：@ident
                        let var = Variable::INT(format!("@{0}", ident));
                        // 将其插入变量表中
                        info.vars_table.insert(ident.clone(), var);
                        // 为该变量进行alloc操作
                        s += &format!("    @{0} = alloc i32\n", ident);
                    }
                }
            }
        }
        (s, Res::Nothing)
    }
}

impl Show for InitVal {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        self.exp.show(info)
    }
}

impl Show for Stmt {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        match self {
            // 对于返回语句，计算返回值并ret即可
            Stmt::Return(exp) => {
                let (sub_exp_str, sub_res) = exp.show(info);
                match sub_res {
                    Res::Nothing => {}
                    Res::Imm => {
                        s += &format!("    ret {}\n", sub_exp_str);
                    }
                    Res::Temp(id) => {
                        s += &sub_exp_str;
                        s += &format!("    ret %{}\n", id);
                    }
                }
                (s, Res::Nothing)
            }

            Stmt::Assign((lval, exp)) => {
                // 首先检查变量是否被定义
                match info.clone().vars_table.get(lval) {
                    Some(var) => {
                        // 检查赋值语句左侧是否是常量
                        match var {
                            // 若是常量，报错
                            Variable::ConstINT(_) => unreachable!(),
                            Variable::INT(ptr_name) => {
                                // 计算右侧表达式
                                let (exp_str, exp_res) = exp.show(info);
                                match exp_res {
                                    Res::Imm => {
                                        s += &format!("    store {0}, {1}\n", exp_str, ptr_name);
                                    }
                                    Res::Temp(idx) => {
                                        s += &exp_str;
                                        s += &format!("    store %{0}, {1}\n", idx, ptr_name);
                                    }
                                    Res::Nothing => unreachable!(),
                                }
                            }
                        }
                    }
                    // 若变量未被定义过，报错
                    None => unreachable!(),
                }
                (s, Res::Nothing)
            }
        }
    }
}

impl Show for Exp {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let (sub_exp_str, sub_flag) = self.lor_exp.show(info);
        s += &sub_exp_str;
        (s, sub_flag)
    }
}

impl Show for UnaryExp {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let mut res = Res::Nothing;
        match self {
            UnaryExp::PrimaryExp(pri_exp) => {
                let (sub_exp_str, sub_res) = pri_exp.show(info);
                s += &sub_exp_str;
                res = sub_res;
            }
            UnaryExp::UnaryExp((unary_op, sub_exp)) => {
                let (sub_exp_str, sub_res) = sub_exp.show(info);
                match unary_op {
                    UnaryOp::Passive => {
                        s += &sub_exp_str;
                        res = sub_res;
                    }
                    UnaryOp::Negative => {
                        s += &sub_exp_str;
                        match sub_res {
                            Res::Nothing => {}
                            Res::Imm => {
                                s = format!("    %{0} = sub 0, {1}\n", info.temp_id, s);
                                res = Res::Temp(info.temp_id);
                                info.temp_id += 1;
                            }
                            Res::Temp(id) => {
                                s += &format!("    %{0} = sub 0, %{1}\n", info.temp_id, id);
                                res = Res::Temp(info.temp_id);
                                info.temp_id += 1;
                            }
                        }
                    }
                    UnaryOp::Inversion => {
                        s += &sub_exp_str;
                        match sub_res {
                            Res::Nothing => {}
                            Res::Imm => {
                                s = format!("    %{0} = eq 0, {1}\n", info.temp_id, s);
                                res = Res::Temp(info.temp_id);
                                info.temp_id += 1;
                            }
                            Res::Temp(id) => {
                                s += &format!("    %{0} = eq 0, %{1}\n", info.temp_id, id);
                                res = Res::Temp(info.temp_id);
                                info.temp_id += 1;
                            }
                        }
                    }
                }
            }
        }
        (s, res)
    }
}

impl Show for PrimaryExp {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let mut res = Res::Nothing;
        match self {
            PrimaryExp::Exp(exp) => {
                let (exp_str, sub_res) = exp.show(info);
                s += &exp_str;
                res = sub_res;
            }
            PrimaryExp::Number(num) => {
                s += &num.to_string();
                res = Res::Imm;
            }
            PrimaryExp::LVal(var) => match info.vars_table.get(var) {
                Some(const_val) => {
                    match const_val {
                        // 对于常量，直接将值代入即可
                        Variable::ConstINT(const_int) => {
                            s += &const_int.to_string();
                            res = Res::Imm;
                        }
                        Variable::INT(ptr_name) => {
                            // 首先将变量load进一个临时变量
                            s += &format!("    %{0} = load {1}\n", info.temp_id, ptr_name);
                            // 返回这个临时变量
                            res = Res::Temp(info.temp_id);
                            info.temp_id += 1;
                        }
                    }
                }
                None => unreachable!(),
            },
        }
        (s, res)
    }
}

impl Show for AddExp {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let mut res = Res::Nothing;
        match self {
            AddExp::AddExp((add_exp, add_op, mul_exp)) => {
                let (add_exp_str, add_exp_res) = add_exp.show(info);
                let (mul_exp_str, mul_exp_res) = mul_exp.show(info);
                // 获取第一个操作数的字符串表示
                let mut op1 = "".to_string();
                match add_exp_res {
                    Res::Nothing => {}
                    Res::Imm => {
                        op1 = add_exp_str;
                    }
                    Res::Temp(id) => {
                        op1 = format!("%{}", id);
                        s += &add_exp_str;
                    }
                }
                // 获取第一个操作数的字符串表示
                let mut op2 = "".to_string();
                match mul_exp_res {
                    Res::Nothing => {}
                    Res::Imm => {
                        op2 = mul_exp_str;
                    }
                    Res::Temp(id) => {
                        op2 = format!("%{}", id);
                        s += &mul_exp_str;
                    }
                }
                // 按照不同运算符生成运算表达式
                match add_op {
                    AddOp::Add => {
                        s += &format!("    %{0} = add {1}, {2}\n", info.temp_id, op1, op2);
                        res = Res::Temp(info.temp_id);
                        info.temp_id += 1;
                    }
                    AddOp::Sub => {
                        s += &format!("    %{0} = sub {1}, {2}\n", info.temp_id, op1, op2);
                        res = Res::Temp(info.temp_id);
                        info.temp_id += 1;
                    }
                }
            }
            AddExp::MulExp(mul_exp) => {
                let (sub_str, sub_res) = mul_exp.show(info);
                s += &sub_str;
                res = sub_res;
            }
        }
        (s, res)
    }
}

impl Show for MulExp {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let mut res = Res::Nothing;
        match self {
            MulExp::UnaryExp(unary_exp) => {
                let (sub_str, sub_res) = unary_exp.show(info);
                s += &sub_str;
                res = sub_res;
            }
            MulExp::MulExp((mul_exp, mul_op, unary_exp)) => {
                let (mul_exp_str, mul_exp_res) = mul_exp.show(info);
                let (unary_exp_str, unary_exp_res) = unary_exp.show(info);
                // 获取第一个操作数的字符串表示
                let mut op1 = "".to_string();
                match mul_exp_res {
                    Res::Nothing => {}
                    Res::Imm => {
                        op1 = mul_exp_str;
                    }
                    Res::Temp(id) => {
                        op1 = format!("%{}", id);
                        s += &mul_exp_str;
                    }
                }
                // 获取第一个操作数的字符串表示
                let mut op2 = "".to_string();
                match unary_exp_res {
                    Res::Nothing => {}
                    Res::Imm => {
                        op2 = unary_exp_str;
                    }
                    Res::Temp(id) => {
                        op2 = format!("%{}", id);
                        s += &unary_exp_str;
                    }
                }
                // 按照不同运算符生成运算表达式
                match mul_op {
                    MulOp::Multiple => {
                        s += &format!("    %{0} = mul {1}, {2}\n", info.temp_id, op1, op2);
                        res = Res::Temp(info.temp_id);
                        info.temp_id += 1;
                    }
                    MulOp::Divide => {
                        s += &format!("    %{0} = div {1}, {2}\n", info.temp_id, op1, op2);
                        res = Res::Temp(info.temp_id);
                        info.temp_id += 1;
                    }
                    MulOp::Mod => {
                        s += &format!("    %{0} = mod {1}, {2}\n", info.temp_id, op1, op2);
                        res = Res::Temp(info.temp_id);
                        info.temp_id += 1;
                    }
                }
            }
        }
        (s, res)
    }
}

impl Show for LOrExp {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let mut res = Res::Nothing;
        match self {
            LOrExp::LAndExp(land_exp) => {
                let (sub_str, sub_res) = land_exp.show(info);
                s += &sub_str;
                res = sub_res;
            }
            LOrExp::LOrExp((lor_exp, land_exp)) => {
                let (lor_exp_str, lor_exp_res) = lor_exp.show(info);
                let (land_exp_str, land_exp_res) = land_exp.show(info);
                // 获取第一个操作数的字符串表示
                let mut op1 = "".to_string();
                match lor_exp_res {
                    Res::Nothing => {}
                    Res::Imm => {
                        op1 = lor_exp_str;
                    }
                    Res::Temp(id) => {
                        op1 = format!("%{}", id);
                        s += &lor_exp_str;
                    }
                }
                // 获取第一个操作数的字符串表示
                let mut op2 = "".to_string();
                match land_exp_res {
                    Res::Nothing => {}
                    Res::Imm => {
                        op2 = land_exp_str;
                    }
                    Res::Temp(id) => {
                        op2 = format!("%{}", id);
                        s += &land_exp_str;
                    }
                }
                s += &format!("    %{0} = or {1}, {2}\n", info.temp_id, op1, op2);
                s += &format!(
                    "    %{0} = ne %{1}, {2}\n",
                    info.temp_id + 1,
                    info.temp_id,
                    0
                );
                res = Res::Temp(info.temp_id + 1);
                info.temp_id += 2;
            }
        }
        (s, res)
    }
}

impl Show for LAndExp {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let mut res = Res::Nothing;
        match self {
            LAndExp::EqExp(land_exp) => {
                let (sub_str, sub_res) = land_exp.show(info);
                s += &sub_str;
                res = sub_res;
            }
            LAndExp::LAndExp((land_exp, eq_exp)) => {
                let (land_exp_str, land_exp_res) = land_exp.show(info);
                let (eq_exp_str, eq_exp_res) = eq_exp.show(info);
                // 获取第一个操作数的字符串表示
                let mut op1 = "".to_string();
                match land_exp_res {
                    Res::Nothing => {}
                    Res::Imm => {
                        op1 = land_exp_str;
                    }
                    Res::Temp(id) => {
                        op1 = format!("%{}", id);
                        s += &land_exp_str;
                    }
                }
                // 获取第一个操作数的字符串表示
                let mut op2 = "".to_string();
                match eq_exp_res {
                    Res::Nothing => {}
                    Res::Imm => {
                        op2 = eq_exp_str;
                    }
                    Res::Temp(id) => {
                        op2 = format!("%{}", id);
                        s += &eq_exp_str;
                    }
                }
                s += &format!("    %{0} = ne {1}, {2}\n", info.temp_id, op1, 0);
                s += &format!("    %{0} = ne {1}, {2}\n", info.temp_id + 1, op2, 0);
                s += &format!(
                    "    %{0} = and %{1}, %{2}\n",
                    info.temp_id + 2,
                    info.temp_id + 1,
                    info.temp_id
                );
                res = Res::Temp(info.temp_id + 2);
                info.temp_id += 3;
            }
        }
        (s, res)
    }
}

impl Show for EqExp {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let mut res = Res::Nothing;
        match self {
            EqExp::RelExp(rel_exp) => {
                let (sub_str, sub_res) = rel_exp.show(info);
                s += &sub_str;
                res = sub_res;
            }
            EqExp::EqExp((eq_exp, comp_op, rel_exp)) => {
                let (eq_exp_str, eq_exp_res) = eq_exp.show(info);
                let (rel_exp_str, rel_exp_res) = rel_exp.show(info);
                // 获取第一个操作数的字符串表示
                let mut op1 = "".to_string();
                match eq_exp_res {
                    Res::Nothing => {}
                    Res::Imm => {
                        op1 = eq_exp_str;
                    }
                    Res::Temp(id) => {
                        op1 = format!("%{}", id);
                        s += &eq_exp_str;
                    }
                }
                // 获取第一个操作数的字符串表示
                let mut op2 = "".to_string();
                match rel_exp_res {
                    Res::Nothing => {}
                    Res::Imm => {
                        op2 = rel_exp_str;
                    }
                    Res::Temp(id) => {
                        op2 = format!("%{}", id);
                        s += &rel_exp_str;
                    }
                }

                let mut op = "".to_string();
                match comp_op {
                    CmpOp::Eq => op += "eq",
                    CmpOp::NEq => op += "ne",
                    _ => unreachable!(),
                }
                s += &format!("    %{0} = {1} {2}, {3}\n", info.temp_id, op, op1, op2);
                res = Res::Temp(info.temp_id);
                info.temp_id += 1;
            }
        }
        (s, res)
    }
}

impl Show for RelExp {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let mut res = Res::Nothing;
        match self {
            RelExp::AddExp(add_exp) => {
                let (sub_str, sub_res) = add_exp.show(info);
                s += &sub_str;
                res = sub_res;
            }
            RelExp::CompExp((rel_exp, comp_op, add_exp)) => {
                let (rel_exp_str, rel_exp_res) = rel_exp.show(info);
                let (add_exp_str, add_exp_res) = add_exp.show(info);
                // 获取第一个操作数的字符串表示
                let mut op1 = "".to_string();
                match rel_exp_res {
                    Res::Nothing => {}
                    Res::Imm => {
                        op1 = rel_exp_str;
                    }
                    Res::Temp(id) => {
                        op1 = format!("%{}", id);
                        s += &rel_exp_str;
                    }
                }
                // 获取第一个操作数的字符串表示
                let mut op2 = "".to_string();
                match add_exp_res {
                    Res::Nothing => {}
                    Res::Imm => {
                        op2 = add_exp_str;
                    }
                    Res::Temp(id) => {
                        op2 = format!("%{}", id);
                        s += &add_exp_str;
                    }
                }

                let mut op = "".to_string();
                match comp_op {
                    CmpOp::Less => op += "lt",
                    CmpOp::Grate => op += "gt",
                    CmpOp::LessEq => op += "le",
                    CmpOp::GrateEq => op += "ge",
                    _ => unreachable!(),
                }
                s += &format!("    %{0} = {1} {2}, {3}\n", info.temp_id, op, op1, op2);
                res = Res::Temp(info.temp_id);
                info.temp_id += 1;
            }
        }
        (s, res)
    }
}
