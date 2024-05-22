use crate::ir_gen::ast::*;
use std::{collections::HashMap, fmt::format};

use super::calc::Calc;

impl CompUnit {
    /// generate koopa ir from a CompUnit in String form
    pub fn generate_koopa(&self) -> String {
        let mut compiler_info = CompilerInfo {
            temp_id: 0,
            vars_table: HashMap::new(),
            field_depth: 0,
            flag_id: 0,
            var_id: 0,
        };
        self.show(&mut compiler_info).0
    }
}

#[derive(Debug, PartialEq, Clone)]
struct CompilerInfo {
    pub temp_id: i32,
    pub vars_table: HashMap<String, (Variable, i32)>,
    pub field_depth: i32,
    pub flag_id: i32,
    pub var_id: i32,
}

enum Res {
    Nothing,
    Imm,
    Temp(i32),
    Ret,
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
        s += "{\n%entry:\n";
        let mut next_info = info.clone();
        next_info.field_depth += 1;
        s += &self.block.show(&mut next_info).0;
        s += "\tret 0\n}\n";
        (s, Res::Nothing)
    }
}

impl Show for Block {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let mut ds = "".to_string();
        let mut res = Res::Nothing;
        for item in &self.items {
            (ds, res) = item.show(info);
            s += &ds;
        }
        (s, Res::Nothing)
    }
}

impl Show for BlockItem {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let mut res = Res::Nothing;
        match self {
            BlockItem::Decl(decl) => {
                s += &*decl.show(info).0;
            }
            BlockItem::Stmt(stmt) => {
                (s, res) = stmt.show(info);
            }
        }
        (s, res)
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
        let mut calculate_info = info
            .clone()
            .vars_table
            .into_iter()
            .map(|(k, (v, _))| (k, v))
            .collect();
        // 在变量表中寻找是否该变量已经被定义
        match info.vars_table.get_mut(&self.ident) {
            // 若未被定义，将其添加进变量表中
            None => {
                let value = self.const_init_val.calculate(&mut calculate_info);
                info.vars_table.insert(
                    self.ident.clone(),
                    (Variable::ConstINT(value), info.field_depth),
                );
            }
            // 否则，判断已定义的变量是否跟当前处于同一作用域
            Some(value) => {
                if value.1 == info.field_depth {
                    // 若处于同一作用域，属于重复定义
                    panic!("变量{:?}重复定义！\n", self.ident);
                } else {
                    // 否则覆盖定义
                    let new_value = self.const_init_val.calculate(&mut calculate_info);
                    // 更新变量表
                    *value = (Variable::ConstINT(new_value), info.field_depth);
                }
            }
        }
        ("".to_string(), Res::Nothing)
    }
}

impl Show for VarDef {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        match self {
            VarDef::Def((ident, init_val)) => {
                let mut vt = info.vars_table.clone();
                // 首先检查该变量是否已被定义。
                match vt.get_mut(ident) {
                    // 若已被定义，检查作用域
                    Some(int) => {
                        if int.1 == info.field_depth {
                            // 若处于同一作用域，属于重复定义
                            panic!("变量{:?}重复定义！\n", ident);
                        } else {
                            // 生成该变量对应的指针的名字：@ident_depth
                            let var_name = format!("@{0}_{1}", ident, info.var_id);
                            info.var_id += 1;
                            // 为该变量进行alloc操作
                            s += &format!("\t{0} = alloc i32\n", var_name);
                            // 计算init_val
                            let (init_s, init_res) = init_val.show(info);
                            // 将结果存进内存
                            match init_res {
                                Res::Nothing => unreachable!(),
                                Res::Imm => {
                                    s += &format!("\tstore {0}, {1}\n", init_s, var_name);
                                }
                                Res::Temp(idx) => {
                                    s += &init_s;
                                    s += &format!("\tstore %{0}, {1}\n", idx, var_name);
                                }
                                _ => {}
                            }
                            let var = Variable::INT(var_name);
                            // 修改变量表
                            info.vars_table
                                .insert(ident.clone(), (var, info.field_depth));
                        }
                    }
                    // 若尚未被定义，将其加入变量表
                    None => {
                        // 生成该变量对应的指针的名字：@ident_depth
                        let var_name = format!("@{0}_{1}", ident, info.var_id);
                        info.var_id += 1;
                        // 为该变量进行alloc操作
                        s += &format!("\t{0} = alloc i32\n", var_name);
                        // 计算init_val
                        let (init_s, init_res) = init_val.show(info);
                        // 将结果存进内存
                        match init_res {
                            Res::Nothing => unreachable!(),
                            Res::Imm => {
                                s += &format!("\tstore {0}, {1}\n", init_s, var_name);
                            }
                            Res::Temp(idx) => {
                                s += &init_s;
                                s += &format!("\tstore %{0}, {1}\n", idx, var_name);
                            }
                            _ => {}
                        }
                        let var = Variable::INT(var_name);
                        // 将其插入变量表中
                        info.vars_table
                            .insert(ident.clone(), (var, info.field_depth));
                    }
                }
            }
            VarDef::Decl(ident) => {
                match info.vars_table.get_mut(ident) {
                    // 若已被定义，检查作用域
                    Some(int) => {
                        if int.1 == info.field_depth {
                            // 若处于同一作用域，属于重复定义
                            panic!("变量{:?}重复定义！\n", ident);
                        } else {
                            // 生成该变量对应的指针的名字：@ident_depth
                            let var_name = format!("@{0}_{1}", ident, info.var_id);
                            info.var_id += 1;
                            // 为该变量进行alloc操作
                            s += &format!("\t{0} = alloc i32\n", var_name);
                            let var = Variable::INT(var_name);
                            // 将其插入变量表中
                            *int = (var, info.field_depth);
                        }
                    }
                    // 若尚未被定义，将其加入变量表
                    None => {
                        // 生成该变量对应的指针的名字：@ident_depth
                        let var_name = format!("@{0}_{1}", ident, info.var_id);
                        info.var_id += 1;
                        // 为该变量进行alloc操作
                        s += &format!("\t{0} = alloc i32\n", var_name);
                        let var = Variable::INT(var_name);
                        // 将其插入变量表中
                        info.vars_table
                            .insert(ident.clone(), (var, info.field_depth));
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

impl Show for If {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let res = Res::Nothing;

        let then_flag = format!("%flag{0}", info.flag_id);
        let else_flag = format!("%flag{0}", info.flag_id + 1);
        let end_flag = format!("%flag{0}", info.flag_id + 2);
        info.flag_id += 3;

        let (cond_str, cond_res) = self.cond.show(info);
        match cond_res {
            Res::Temp(temp_id) => {
                s += &cond_str;
                s += &format!("\tbr %{0}, {1}, {2}\n", temp_id, then_flag, else_flag);
            }
            Res::Imm => {
                s += &format!(
                    "\t%{0} = add {1}, 0\n\tbr %{0}, {2}, {3}\n",
                    info.temp_id, cond_str, then_flag, else_flag
                );
                info.temp_id += 1;
            }
            _ => unreachable!(),
        }

        s += &(then_flag + ":\n");
        let (then_str, then_res) = self.then_stmt.show(info);
        s += &(then_str + "\tjump " + &end_flag + "\n");

        s += &(else_flag + ":\n");
        match &self.else_stmt {
            None => {
                s += &format!("\tjump {0}\n", end_flag);
            }
            Some(stmt) => {
                let (else_str, else_res) = stmt.show(info);
                s += &else_str;
                s += &format!("\tjump {0}\n", end_flag);
            }
        }

        s += &(end_flag + ":\n");

        (s, res)
    }
}

impl Show for Stmt {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        match self {
            // 对于返回语句，计算返回值并ret即可
            Stmt::Return(exp) => {
                match exp {
                    None => {
                        s += &format!("\tret\n%flag{0}:\n", info.flag_id);
                        info.flag_id += 1;
                    }
                    Some(e) => {
                        let (sub_exp_str, sub_res) = e.show(info);
                        match sub_res {
                            Res::Nothing => {}
                            Res::Imm => {
                                s += &format!("\tret {0}\n%flag{1}:\n", sub_exp_str, info.flag_id);
                                info.flag_id += 1;
                            }
                            Res::Temp(id) => {
                                s += &sub_exp_str;
                                s += &format!("\tret %{0}\n%flag{1}:\n", id, info.flag_id);
                                info.flag_id += 1;
                            }
                            _ => {}
                        }
                    }
                }
                (s, Res::Ret)
            }

            Stmt::Assign((lval, exp)) => {
                // 首先检查变量是否被定义
                match info.clone().vars_table.get(lval) {
                    Some(var) => {
                        // 检查赋值语句左侧是否是常量
                        match &var.0 {
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
                                    _ => {}
                                }
                            }
                        }
                    }
                    // 若变量未被定义过，报错
                    None => unreachable!(),
                }
                (s, Res::Nothing)
            }
            Stmt::Block(block) => {
                let mut next_info = info.clone();
                next_info.field_depth += 1;
                s += &block.show(&mut next_info).0;
                info.temp_id = next_info.temp_id;
                info.flag_id = next_info.flag_id;
                info.var_id = next_info.var_id;
                (s, Res::Nothing)
            }
            Stmt::Exp(exp) => {
                match exp {
                    None => {}
                    Some(e) => {
                        let (exp_str, exp_res) = e.show(info);
                        match exp_res {
                            Res::Temp(temp_id) => {
                                s += &exp_str;
                            }
                            Res::Imm => {}
                            _ => unreachable!(),
                        }
                    }
                }
                (s, Res::Nothing)
            }
            Stmt::IF(if_stmt) => {
                s += &if_stmt.show(info).0;
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
                            _ => {}
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
                            _ => {}
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
            PrimaryExp::LVal(var) => match info.vars_table.get_mut(var) {
                Some(const_val) => {
                    match &const_val.0 {
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
                    _ => {}
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
                    _ => {}
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
                    _ => {}
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
                    _ => {}
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
                    _ => {}
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
                    _ => {}
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
                    _ => {}
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
                    _ => {}
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
                    _ => {}
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
                    _ => {}
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
                    _ => {}
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
                    _ => {}
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
