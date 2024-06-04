use super::calc::Calc;
use crate::ir_gen::ast::*;
use std::collections::HashMap;

fn global_zero_array(dims: &Vec<i32>) -> String {
    let mut s = "".to_string();
    s += "{";
    for i in 0..dims[0] {
        if i > 0 {
            s += ", ";
        }
        if dims.len() == 1 {
            s += "0";
        } else {
            s += &global_zero_array(&dims[1..].to_vec());
        }
    }
    s += "}";
    s
}
fn local_zero_array(info: &mut CompilerInfo, dims: &Vec<i32>, base_ptr: String) -> String {
    let mut s = "".to_string();
    for i in 0..dims[0] {
        if dims.len() == 1 {
            s += &format!(
                "\t%{0} = getelemptr {1}, {2}\n\tstore 0, %{0}\n",
                info.temp_id, base_ptr, i
            );
            info.temp_id += 1;
        } else {
            s += &format!("\t%{0} = getelemptr {1}, {2}\n", info.temp_id, base_ptr, i);
            let sub_base = format!("%{0}", info.temp_id);
            info.temp_id += 1;
            s += &local_zero_array(info, &dims[1..].to_vec(), sub_base);
        }
    }
    s
}

impl CompUnit {
    /// generate koopa ir from a CompUnit in String form
    pub fn generate_koopa(&self) -> String {
        let mut compiler_info = CompilerInfo {
            temp_id: 0,
            vars_table: HashMap::new(),
            field_depth: 0,
            flag_id: 0,
            var_id: 0,
            enter_flag: -1,
            end_flag: -1,
            func_param: false,
            func_name: "".to_string(),
        };

        let var_getint = Variable::Func(("@getint".to_string(), ItemType::Int, vec![]));
        compiler_info
            .vars_table
            .insert("getint".to_string(), (var_getint, 0));

        let var_getch = Variable::Func(("@getch".to_string(), ItemType::Int, vec![]));
        compiler_info
            .vars_table
            .insert("getch".to_string(), (var_getch, 0));

        let var_getarray = Variable::Func(("@getarray".to_string(), ItemType::Int, vec![true]));
        compiler_info
            .vars_table
            .insert("getarray".to_string(), (var_getarray, 0));

        let var_putint = Variable::Func(("@putint".to_string(), ItemType::Void, vec![false]));
        compiler_info
            .vars_table
            .insert("putint".to_string(), (var_putint, 0));

        let var_putch = Variable::Func(("@putch".to_string(), ItemType::Void, vec![false]));
        compiler_info
            .vars_table
            .insert("putch".to_string(), (var_putch, 0));

        let var_putarray =
            Variable::Func(("@putarray".to_string(), ItemType::Void, vec![false, true]));
        compiler_info
            .vars_table
            .insert("putarray".to_string(), (var_putarray, 0));

        let var_starttime = Variable::Func(("@starttime".to_string(), ItemType::Void, vec![]));
        compiler_info
            .vars_table
            .insert("starttime".to_string(), (var_starttime, 0));

        let var_stoptime = Variable::Func(("@stoptime".to_string(), ItemType::Void, vec![]));
        compiler_info
            .vars_table
            .insert("stoptime".to_string(), (var_stoptime, 0));

        let mut s = "decl @getint(): i32\ndecl @getch(): i32\ndecl @getarray(*i32): i32\ndecl @putint(i32)\ndecl @putch(i32)\ndecl @putarray(i32, *i32)\ndecl @starttime()\ndecl @stoptime()\n\n".to_string();
        s += &self.show(&mut compiler_info).0;
        s
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CompilerInfo {
    pub temp_id: i32,
    /// var_ident, (variable, depth)
    pub vars_table: HashMap<String, (Variable, i32)>,
    pub field_depth: i32,
    pub flag_id: i32,
    pub var_id: i32,
    pub enter_flag: i32,
    pub end_flag: i32,
    pub func_param: bool,
    pub func_name: String,
}

enum Res {
    Nothing,
    Imm(i32),
    Temp(i32),
    Ret,
    Params(String),
    PtrFlag(Vec<bool>),
}

trait Show {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res);
}

impl Show for CompUnit {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let res = Res::Nothing;
        match &*self.comp_unit {
            Some(sub_comp_unit) => s += &sub_comp_unit.show(info).0,
            None => {}
        }
        match &self.global_item {
            GlobalItem::Func(func_def) => {
                let (_func_str, func_res) = func_def.pre_show(info);
                let mut ptr_flags: Vec<bool> = Vec::new();
                match func_res {
                    Res::PtrFlag(flags) => {
                        ptr_flags = flags.clone();
                    }
                    _ => unreachable!(),
                }
                let fun_var = Variable::Func((
                    format!("@{0}", func_def.id),
                    func_def.func_type.clone(),
                    ptr_flags,
                ));
                info.vars_table.insert(func_def.id.clone(), (fun_var, 0));
                let (func_str, func_res) = func_def.show(info);
                s += &func_str;
            }
            GlobalItem::Decl(decl) => {
                s += &decl.global_show(info);
            }
            _ => unreachable!(),
        }
        (s, res)
    }
}

impl FuncDef {
    fn pre_show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "fun @".to_string();
        let mut res = Res::PtrFlag(Vec::new());
        match &self.func_f_params {
            None => {
                s += "()";
            }
            Some(func_f_params) => {
                let (sub_str, sub_res) = func_f_params.show(info);
                s += &format!("({0})", sub_str);
                res = sub_res;
            }
        }
        (s, res)
    }
}

impl Show for FuncDef {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "fun @".to_string();
        let mut res = Res::PtrFlag(Vec::new());
        s += &self.id;
        match &self.func_f_params {
            None => {
                s += "()";
            }
            Some(func_f_params) => {
                let (sub_str, sub_res) = func_f_params.show(info);
                s += &format!("({0})", sub_str);
                res = sub_res;
            }
        }
        match self.func_type {
            ItemType::Int => {
                s += ": i32";
            }
            ItemType::Void => {}
            _ => unreachable!(),
        }
        s += "{\n%entry:\n";
        let mut next_info = info.clone();
        next_info.field_depth += 1;
        match &self.func_f_params {
            None => {}
            Some(func_f_params) => {
                s += &func_f_params.allocate_for_params(&mut next_info);
            }
        }
        s += &self.block.show(&mut next_info).0;
        s += "\tret\n}\n";
        (s, res)
    }
}

impl Show for FuncFParams {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let mut flags: Vec<bool> = Vec::new();
        let mut i = 0;
        for param in &self.func_f_params {
            match &param.dims {
                None => {
                    flags.push(false);
                    if i == 0 {
                        s += &format!("@{0}: i32", param.id);
                    } else {
                        s += &format!(", @{0}: i32", param.id);
                    }
                }
                Some(indices) => {
                    flags.push(true);
                    let mut calculate_info = info
                        .clone()
                        .vars_table
                        .into_iter()
                        .map(|(k, (v, _))| (k, v))
                        .collect();

                    let mut dims: Vec<i32> = Vec::new();
                    let mut array_str = "i32".to_string();
                    for dim in indices {
                        dims.push(dim.calculate(&mut calculate_info));
                    }
                    for dim in dims.iter().rev() {
                        array_str = format!("[{0}, {1}]", array_str, dim);
                    }

                    if i == 0 {
                        s += &format!("@{0}: *{1}", param.id, array_str);
                    } else {
                        s += &format!(", @{0}: *{1}", param.id, array_str);
                    }
                }
            }
            i += 1;
        }
        (s, Res::PtrFlag(flags))
    }
}

impl FuncFParams {
    pub fn allocate_for_params(&self, info: &mut CompilerInfo) -> String {
        let mut s = "".to_string();
        for param in &self.func_f_params {
            match &param.dims {
                None => {
                    let var = Variable::INT(format!("%{0}", param.id));
                    info.vars_table
                        .insert(param.id.clone(), (var, info.field_depth));
                    s += &format!("\t%{0} = alloc i32\n\tstore @{0}, %{0}\n", param.id);
                }
                Some(indices) => {
                    let mut calculate_info = info
                        .clone()
                        .vars_table
                        .into_iter()
                        .map(|(k, (v, _))| (k, v))
                        .collect();

                    let mut dims: Vec<i32> = Vec::new();
                    let mut array_str = "i32".to_string();
                    for dim in indices {
                        dims.push(dim.calculate(&mut calculate_info));
                    }
                    for dim in dims.iter().rev() {
                        array_str = format!("[{0}, {1}]", array_str, dim);
                    }

                    let mut var = Variable::Ptr((format!("%{0}", param.id), ItemType::Int));
                    info.vars_table
                        .insert(param.id.clone(), (var, info.field_depth));
                    s += &format!(
                        "\t%{1} = alloc *{0}\n\tstore @{1}, %{1}\n",
                        array_str, param.id
                    );
                }
            }
        }
        s
    }
}

impl Show for FuncRParams {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let mut pre_s = "".to_string();
        let mut i = 0;
        let mut ptr_flags: Vec<bool> = Vec::new();
        match info.vars_table.get(&info.func_name) {
            None => unreachable!(),
            Some((func, depth)) => match func {
                Variable::Func((func_name, item_type, flags)) => {
                    ptr_flags = flags.clone();
                }
                _ => unreachable!(),
            },
        }
        // println!("{:?}", ptr_flags);
        for exp in &self.func_r_params {
            info.func_param = ptr_flags[i];
            let (exp_str, exp_res) = exp.show(info);
            info.func_param = false;
            match exp_res {
                Res::Imm(imm) => {
                    if i == 0 {
                        s += &exp_str;
                    } else {
                        s += &format!(", {0}", exp_str);
                    }
                }
                Res::Temp(temp) => {
                    pre_s += &exp_str;
                    if i == 0 {
                        s += &format!("%{0}", temp);
                    } else {
                        s += &format!(", %{0}", temp);
                    }
                }
                _ => unreachable!(),
            }
            i += 1;
        }
        (pre_s, Res::Params(s))
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
        (s, res)
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
        let mut s = "".to_string();
        match self.b_type {
            ItemType::Int => {}
            _ => unreachable!(),
        }
        for const_def in &self.const_defs {
            s += &const_def.show(info).0;
        }
        (s, Res::Nothing)
    }
}

impl Show for ConstDef {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let mut calculate_info = info
            .clone()
            .vars_table
            .into_iter()
            .map(|(k, (v, _))| (k, v))
            .collect();
        // 首先判断该定义是常量定义还是数组定义
        if self.dims.len() == 0 {
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
        } else {
            match info.vars_table.get_mut(&self.ident) {
                None => {
                    let mut dims: Vec<i32> = Vec::new();
                    let mut array_str = "i32".to_string();
                    for dim in &self.dims {
                        dims.push(dim.calculate(&mut calculate_info));
                    }
                    for dim in dims.iter().rev() {
                        array_str = format!("[{0}, {1}]", array_str, dim);
                    }
                    let mut val_idx = 0;
                    let init_str = self.const_init_val.local_array_init(
                        info,
                        &dims,
                        &mut val_idx,
                        format!("@{0}", self.ident),
                    );
                    s += &format!("\t@{0} = alloc {1}\n", self.ident, array_str);
                    s += &init_str;
                    let new_var = Variable::Array(format!("@{0}", self.ident));
                    info.vars_table.insert(self.ident.clone(), (new_var, 0));
                }
                Some(value) => {
                    panic!("define a global const variable more than once!");
                }
            }
        }
        (s, Res::Nothing)
    }
}

impl Show for VarDef {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        match self {
            VarDef::Def((ident, dims, init_val)) => {
                if dims.len() == 0 {
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
                                    Res::Imm(imm) => {
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
                                Res::Imm(imm) => {
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
                } else {
                    let mut vt = info.vars_table.clone();
                    // 首先检查该变量是否已被定义。
                    match vt.get_mut(ident) {
                        // 若已被定义，检查作用域
                        Some(int) => {
                            if int.1 == info.field_depth {
                                // 若处于同一作用域，属于重复定义
                                panic!("变量{:?}重复定义！\n", ident);
                            } else {
                                let mut calculate_info = info
                                    .clone()
                                    .vars_table
                                    .into_iter()
                                    .map(|(k, (v, _))| (k, v))
                                    .collect();

                                // 计算数组维度信息
                                let mut dims_i32: Vec<i32> = Vec::new();
                                let mut array_str = "i32".to_string();
                                for dim in dims {
                                    dims_i32.push(dim.calculate(&mut calculate_info));
                                }
                                for dim in dims_i32.iter().rev() {
                                    array_str = format!("[{0}, {1}]", array_str, dim);
                                }

                                // 生成该变量对应的指针的名字：@ident_depth
                                let var_name = format!("@{0}_{1}", ident, info.var_id);
                                info.var_id += 1;
                                // 为该变量进行alloc操作
                                s += &format!("\t{0} = alloc {1}\n", var_name, array_str);
                                // 初始化
                                let mut val_idx = 0;
                                let init_s = init_val.local_array_init(
                                    info,
                                    &dims_i32,
                                    &mut val_idx,
                                    var_name.clone(),
                                );
                                s += &init_s;
                                let var = Variable::Array(var_name);
                                // 修改变量表
                                info.vars_table
                                    .insert(ident.clone(), (var, info.field_depth));
                            }
                        }
                        // 若尚未被定义，将其加入变量表
                        None => {
                            let mut calculate_info = info
                                .clone()
                                .vars_table
                                .into_iter()
                                .map(|(k, (v, _))| (k, v))
                                .collect();

                            // 计算数组维度信息
                            let mut dims_i32: Vec<i32> = Vec::new();
                            let mut array_str = "i32".to_string();
                            for dim in dims {
                                dims_i32.push(dim.calculate(&mut calculate_info));
                            }
                            for dim in dims_i32.iter().rev() {
                                array_str = format!("[{0}, {1}]", array_str, dim);
                            }

                            // 生成该变量对应的指针的名字：@ident_depth
                            let var_name = format!("@{0}_{1}", ident, info.var_id);
                            info.var_id += 1;
                            // 为该变量进行alloc操作
                            s += &format!("\t{0} = alloc {1}\n", var_name, array_str);
                            // 计算init_val
                            let mut val_idx = 0;
                            let init_s = init_val.local_array_init(
                                info,
                                &dims_i32,
                                &mut val_idx,
                                var_name.clone(),
                            );
                            s += &init_s;
                            let var = Variable::Array(var_name);
                            // 将其插入变量表中
                            info.vars_table
                                .insert(ident.clone(), (var, info.field_depth));
                        }
                    }
                }
            }
            VarDef::Decl((ident, dims)) => {
                if dims.len() == 0 {
                    match info.vars_table.get_mut(&ident.clone()) {
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
                } else {
                    let mut vt = info.vars_table.clone();
                    // 首先检查该变量是否已被定义。
                    match vt.get_mut(ident) {
                        // 若已被定义，检查作用域
                        Some(int) => {
                            if int.1 == info.field_depth {
                                // 若处于同一作用域，属于重复定义
                                panic!("变量{:?}重复定义！\n", ident);
                            } else {
                                let mut calculate_info = info
                                    .clone()
                                    .vars_table
                                    .into_iter()
                                    .map(|(k, (v, _))| (k, v))
                                    .collect();

                                // 计算数组维度信息
                                let mut dims_i32: Vec<i32> = Vec::new();
                                let mut array_str = "i32".to_string();
                                for dim in dims {
                                    dims_i32.push(dim.calculate(&mut calculate_info));
                                }
                                for dim in dims_i32.iter().rev() {
                                    array_str = format!("[{0}, {1}]", array_str, dim);
                                }

                                // 生成该变量对应的指针的名字：@ident_depth
                                let var_name = format!("@{0}_{1}", ident, info.var_id);
                                info.var_id += 1;
                                // 为该变量进行alloc操作
                                s += &format!("\t{0} = alloc {1}\n", var_name, array_str);
                                let var = Variable::Array(var_name);
                                // 修改变量表
                                info.vars_table
                                    .insert(ident.clone(), (var, info.field_depth));
                            }
                        }
                        // 若尚未被定义，将其加入变量表
                        None => {
                            let mut calculate_info = info
                                .clone()
                                .vars_table
                                .into_iter()
                                .map(|(k, (v, _))| (k, v))
                                .collect();

                            // 计算数组维度信息
                            let mut dims_i32: Vec<i32> = Vec::new();
                            let mut array_str = "i32".to_string();
                            for dim in dims {
                                dims_i32.push(dim.calculate(&mut calculate_info));
                            }
                            for dim in dims_i32.iter().rev() {
                                array_str = format!("[{0}, {1}]", array_str, dim);
                            }

                            // 生成该变量对应的指针的名字：@ident_depth
                            let var_name = format!("@{0}_{1}", ident, info.var_id);
                            info.var_id += 1;
                            // 为该变量进行alloc操作
                            s += &format!("\t{0} = alloc {1}\n", var_name, array_str);
                            let var = Variable::Array(var_name);
                            // 将其插入变量表中
                            info.vars_table
                                .insert(ident.clone(), (var, info.field_depth));
                        }
                    }
                }
            }
        }
        (s, Res::Nothing)
    }
}

impl Show for InitVal {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        match self {
            InitVal::Exp(exp) => exp.show(info),
            InitVal::Array(array) => ("".to_string(), Res::Nothing),
        }
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
            Res::Imm(imm) => {
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

impl Show for While {
    fn show(&self, info: &mut CompilerInfo) -> (String, Res) {
        let mut s = "".to_string();
        let res = Res::Nothing;

        let enter_flag = format!("%flag{0}", info.flag_id);
        let body_flag = format!("%flag{0}", info.flag_id + 1);
        let end_flag = format!("%flag{0}", info.flag_id + 2);
        info.flag_id += 3;
        s += &format!("{0}:\n", enter_flag);
        let (cond_str, cond_res) = self.cond.show(info);
        match cond_res {
            Res::Temp(temp_id) => {
                s += &cond_str;
                s += &format!("\tbr %{0}, {1}, {2}\n", temp_id, body_flag, end_flag);
            }
            Res::Imm(imm) => {
                s += &format!(
                    "\t%{0} = add {1}, 0\n\tbr %{0}, {2}, {3}\n",
                    info.temp_id, cond_str, body_flag, end_flag
                );
                info.temp_id += 1;
            }
            _ => unreachable!(),
        }
        s += &(body_flag + ":\n");
        let ori_enter_flag = info.enter_flag;
        let ori_end_flag = info.end_flag;
        info.enter_flag = info.flag_id - 3;
        info.end_flag = info.flag_id - 1;
        let (body_str, body_res) = self.body_stmt.show(info);
        info.enter_flag = ori_enter_flag;
        info.end_flag = ori_end_flag;
        s += &body_str;
        s += &format!("\tjump {0}\n", enter_flag);
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
                            Res::Imm(imm) => {
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
                let mut indices: Vec<Res> = Vec::new();
                for index in &lval.indices {
                    let (ind_str, ind_res) = index.show(info);
                    match ind_res {
                        Res::Imm(imm) => indices.push(Res::Imm(imm)),
                        Res::Temp(temp) => {
                            s += &ind_str;
                            indices.push(Res::Temp(temp));
                        }
                        _ => unreachable!(),
                    }
                }
                // 首先检查变量是否被定义
                match info.clone().vars_table.get(&lval.ident) {
                    Some(var) => {
                        // 检查赋值语句左侧是否是常量
                        match &var.0 {
                            // 若是常量，报错
                            Variable::ConstINT(_) => unreachable!(),
                            Variable::INT(ptr_name) => {
                                // 计算右侧表达式
                                let (exp_str, exp_res) = exp.show(info);
                                match exp_res {
                                    Res::Imm(imm) => {
                                        s += &format!("\tstore {0}, {1}\n", exp_str, ptr_name);
                                    }
                                    Res::Temp(idx) => {
                                        s += &exp_str;
                                        s += &format!("\tstore %{0}, {1}\n", idx, ptr_name);
                                    }
                                    Res::Nothing => unreachable!(),
                                    _ => {}
                                }
                            }
                            Variable::Array(array_name) => {
                                let mut source_addr = array_name.clone();
                                for index in indices {
                                    match index {
                                        Res::Imm(imm) => {
                                            s += &format!(
                                                "\t%{0} = getelemptr {1}, {2}\n",
                                                info.temp_id, source_addr, imm
                                            );
                                            source_addr = format!("%{0}", info.temp_id);
                                            info.temp_id += 1;
                                        }
                                        Res::Temp(temp) => {
                                            s += &format!(
                                                "\t%{0} = getelemptr {1}, %{2}\n",
                                                info.temp_id, source_addr, temp
                                            );
                                            source_addr = format!("%{0}", info.temp_id);
                                            info.temp_id += 1;
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                let (exp_str, exp_res) = exp.show(info);
                                match exp_res {
                                    Res::Imm(imm) => {
                                        s += &format!("\tstore {0}, {1}\n", exp_str, source_addr);
                                    }
                                    Res::Temp(idx) => {
                                        s += &exp_str;
                                        s += &format!("\tstore %{0}, {1}\n", idx, source_addr);
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            Variable::Ptr((ptr_name, tar_type)) => {
                                let mut source_addr = ptr_name.clone();
                                let mut flag = 0;
                                s += &format!("\t%{0} = load {1}\n", info.temp_id, source_addr);
                                source_addr = format!("%{0}", info.temp_id);
                                info.temp_id += 1;
                                for index in indices {
                                    match index {
                                        Res::Imm(imm) => {
                                            if flag == 0 {
                                                s += &format!(
                                                    "\t%{0} = getptr {1}, {2}\n",
                                                    info.temp_id, source_addr, imm
                                                );
                                                flag += 1;
                                            } else {
                                                s += &format!(
                                                    "\t%{0} = getelemptr {1}, {2}\n",
                                                    info.temp_id, source_addr, imm
                                                );
                                            }
                                            source_addr = format!("%{0}", info.temp_id);
                                            info.temp_id += 1;
                                        }
                                        Res::Temp(temp) => {
                                            if flag == 0 {
                                                s += &format!(
                                                    "\t%{0} = getptr {1}, %{2}\n",
                                                    info.temp_id, source_addr, temp
                                                );
                                                flag += 1;
                                            } else {
                                                s += &format!(
                                                    "\t%{0} = getelemptr {1}, %{2}\n",
                                                    info.temp_id, source_addr, temp
                                                );
                                            }
                                            source_addr = format!("%{0}", info.temp_id);
                                            info.temp_id += 1;
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                let (exp_str, exp_res) = exp.show(info);
                                match exp_res {
                                    Res::Imm(imm) => {
                                        s += &format!("\tstore {0}, {1}\n", exp_str, source_addr);
                                    }
                                    Res::Temp(idx) => {
                                        s += &exp_str;
                                        s += &format!("\tstore %{0}, {1}\n", idx, source_addr);
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            _ => unreachable!(),
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
                let (blk_str, blk_res) = block.show(&mut next_info);
                s += &blk_str;
                info.temp_id = next_info.temp_id;
                info.flag_id = next_info.flag_id;
                info.var_id = next_info.var_id;
                (s, blk_res)
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
                            Res::Imm(imm) => {}
                            Res::Nothing => {
                                s += &exp_str;
                            }
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
            Stmt::WHILE(while_stmt) => {
                s += &format!("\tjump %flag{0}\n", info.flag_id);
                s += &while_stmt.show(info).0;
                (s, Res::Nothing)
            }
            Stmt::Break => {
                s += &format!("\tjump %flag{0}\n%flag{1}:\n", info.end_flag, info.flag_id);
                info.flag_id += 1;
                (s, Res::Ret)
            }
            Stmt::Continue => {
                s += &format!(
                    "\tjump %flag{0}\n%flag{1}:\n",
                    info.enter_flag, info.flag_id
                );
                info.flag_id += 1;
                (s, Res::Ret)
            }
            _ => unreachable!(),
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
                            Res::Imm(imm) => {
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
                            Res::Imm(imm) => {
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
            UnaryExp::FuncItem((id, func_r_params)) => {
                let mut func_r_params_str = "".to_string();
                match func_r_params {
                    Some(func_params) => {
                        info.func_name = id.clone();
                        let (param_str, param_res) = func_params.show(info);
                        s += &param_str;
                        match param_res {
                            Res::Params(pstr) => {
                                func_r_params_str += &pstr;
                            }
                            _ => unreachable!(),
                        }
                    }
                    None => {}
                }
                match info.vars_table.get(id) {
                    Some((func, depth)) => match func {
                        Variable::Func((func_name, func_type, ptr_flags)) => {
                            match func_type {
                                ItemType::Int => {
                                    s += &format!("\t%{0} = ", info.temp_id);
                                    res = Res::Temp(info.temp_id);
                                    info.temp_id += 1;
                                }
                                ItemType::Void => {
                                    s += "\t";
                                }
                                _ => unreachable!(),
                            }
                            s += &format!("call {0}({1})\n", func_name, func_r_params_str);
                        }
                        _ => unreachable!(),
                    },
                    None => unreachable!(),
                }
            }
            _ => unreachable!(),
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
                res = Res::Imm(*num);
            }
            PrimaryExp::LVal(var) => {
                let mut indices: Vec<Res> = Vec::new();
                for index in &var.indices {
                    let (ind_str, ind_res) = index.show(info);
                    match ind_res {
                        Res::Imm(imm) => indices.push(Res::Imm(imm)),
                        Res::Temp(temp) => {
                            s += &ind_str;
                            indices.push(Res::Temp(temp));
                        }
                        _ => unreachable!(),
                    }
                }
                match info.vars_table.get_mut(&var.ident) {
                    Some(const_val) => {
                        match &const_val.0 {
                            // 对于常量，直接将值代入即可
                            Variable::ConstINT(const_int) => {
                                s += &const_int.to_string();
                                res = Res::Imm(*const_int);
                            }
                            Variable::INT(ptr_name) => {
                                // 首先将变量load进一个临时变量
                                s += &format!("\t%{0} = load {1}\n", info.temp_id, ptr_name);
                                // 返回这个临时变量
                                res = Res::Temp(info.temp_id);
                                info.temp_id += 1;
                            }
                            Variable::Array(array_name) => {
                                if info.func_param {
                                    let mut source_addr = array_name.clone();
                                    for index in indices {
                                        match index {
                                            Res::Imm(imm) => {
                                                s += &format!(
                                                    "\t%{0} = getelemptr {1}, {2}\n",
                                                    info.temp_id, source_addr, imm
                                                );
                                                source_addr = format!("%{0}", info.temp_id);
                                                info.temp_id += 1;
                                            }
                                            Res::Temp(temp) => {
                                                s += &format!(
                                                    "\t%{0} = getelemptr {1}, %{2}\n",
                                                    info.temp_id, source_addr, temp
                                                );
                                                source_addr = format!("%{0}", info.temp_id);
                                                info.temp_id += 1;
                                            }
                                            _ => unreachable!(),
                                        }
                                    }
                                    s += &format!(
                                        "\t%{0} = getelemptr {1}, 0\n",
                                        info.temp_id, source_addr
                                    );
                                    res = Res::Temp(info.temp_id);
                                    info.temp_id += 1;
                                } else {
                                    let mut source_addr = array_name.clone();
                                    for index in indices {
                                        match index {
                                            Res::Imm(imm) => {
                                                s += &format!(
                                                    "\t%{0} = getelemptr {1}, {2}\n",
                                                    info.temp_id, source_addr, imm
                                                );
                                                source_addr = format!("%{0}", info.temp_id);
                                                info.temp_id += 1;
                                            }
                                            Res::Temp(temp) => {
                                                s += &format!(
                                                    "\t%{0} = getelemptr {1}, %{2}\n",
                                                    info.temp_id, source_addr, temp
                                                );
                                                source_addr = format!("%{0}", info.temp_id);
                                                info.temp_id += 1;
                                            }
                                            _ => unreachable!(),
                                        }
                                    }
                                    s += &format!("\t%{0} = load {1}\n", info.temp_id, source_addr);
                                    res = Res::Temp(info.temp_id);
                                    info.temp_id += 1;
                                }
                            }
                            Variable::Ptr((ptr_name, tar_type)) => {
                                if info.func_param {
                                    let mut source_addr = ptr_name.clone();
                                    s += &format!("\t%{0} = load {1}\n", info.temp_id, source_addr);
                                    source_addr = format!("%{0}", info.temp_id);
                                    info.temp_id += 1;
                                    let mut flag = 0;
                                    for index in indices {
                                        match index {
                                            Res::Imm(imm) => {
                                                if flag == 0 {
                                                    s += &format!(
                                                        "\t%{0} = getptr {1}, {2}\n",
                                                        info.temp_id, source_addr, imm
                                                    );
                                                    flag += 1;
                                                } else {
                                                    s += &format!(
                                                        "\t%{0} = getelemptr {1}, {2}\n",
                                                        info.temp_id, source_addr, imm
                                                    );
                                                }
                                                source_addr = format!("%{0}", info.temp_id);
                                                info.temp_id += 1;
                                            }
                                            Res::Temp(temp) => {
                                                if flag == 0 {
                                                    s += &format!(
                                                        "\t%{0} = getptr {1}, %{2}\n",
                                                        info.temp_id, source_addr, temp
                                                    );
                                                    flag += 1;
                                                } else {
                                                    s += &format!(
                                                        "\t%{0} = getelemptr {1}, %{2}\n",
                                                        info.temp_id, source_addr, temp
                                                    );
                                                }
                                                source_addr = format!("%{0}", info.temp_id);
                                                info.temp_id += 1;
                                            }
                                            _ => unreachable!(),
                                        }
                                    }
                                    if flag > 0 {
                                        s += &format!(
                                            "\t%{0} = getelemptr {1}, 0\n",
                                            info.temp_id, source_addr
                                        );
                                        res = Res::Temp(info.temp_id);
                                        info.temp_id += 1;
                                    } else {
                                        res = Res::Temp(info.temp_id - 1);
                                    }
                                } else {
                                    let mut source_addr = ptr_name.clone();
                                    s += &format!("\t%{0} = load {1}\n", info.temp_id, source_addr);
                                    source_addr = format!("%{0}", info.temp_id);
                                    info.temp_id += 1;
                                    let mut flag = 0;
                                    for index in indices {
                                        match index {
                                            Res::Imm(imm) => {
                                                if flag == 0 {
                                                    s += &format!(
                                                        "\t%{0} = getptr {1}, {2}\n",
                                                        info.temp_id, source_addr, imm
                                                    );
                                                    flag += 1;
                                                } else {
                                                    s += &format!(
                                                        "\t%{0} = getelemptr {1}, {2}\n",
                                                        info.temp_id, source_addr, imm
                                                    );
                                                }
                                                source_addr = format!("%{0}", info.temp_id);
                                                info.temp_id += 1;
                                            }
                                            Res::Temp(temp) => {
                                                if flag == 0 {
                                                    s += &format!(
                                                        "\t%{0} = getptr {1}, %{2}\n",
                                                        info.temp_id, source_addr, temp
                                                    );
                                                    flag += 1;
                                                } else {
                                                    s += &format!(
                                                        "\t%{0} = getelemptr {1}, %{2}\n",
                                                        info.temp_id, source_addr, temp
                                                    );
                                                }
                                                source_addr = format!("%{0}", info.temp_id);
                                                info.temp_id += 1;
                                            }
                                            _ => unreachable!(),
                                        }
                                    }
                                    s += &format!("\t%{0} = load {1}\n", info.temp_id, source_addr);
                                    res = Res::Temp(info.temp_id);
                                    info.temp_id += 1;
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    None => unreachable!(),
                }
            }
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
                    Res::Imm(imm) => {
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
                    Res::Imm(imm) => {
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
                    Res::Imm(imm) => {
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
                    Res::Imm(imm) => {
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
                let true_flag = info.flag_id;
                let false_flag = info.flag_id + 1;
                let end_flag = info.flag_id + 2;
                info.flag_id += 3;
                let (lor_exp_str, lor_exp_res) = lor_exp.show(info);
                let (land_exp_str, land_exp_res) = land_exp.show(info);
                let ans_val = info.var_id;
                s += &format!("\t@bool{0} = alloc i32\n", ans_val);
                info.var_id += 1;
                // 获取第一个操作数的字符串表示
                let mut op1 = "".to_string();
                match lor_exp_res {
                    Res::Imm(imm) => {
                        op1 = lor_exp_str;
                        s += &format!("\tbr {0}, %flag{1}, %flag{2}\n", op1, true_flag, false_flag);
                    }
                    Res::Temp(id) => {
                        op1 = format!("%{}", id);
                        s += &lor_exp_str;
                        s += &format!("\tbr %{0}, %flag{1}, %flag{2}\n", id, true_flag, false_flag);
                    }
                    _ => unreachable!(),
                }
                s += &format!("%flag{0}:\n", false_flag);

                // 获取第二个操作数的字符串表示
                let mut op2 = "".to_string();
                match land_exp_res {
                    Res::Imm(imm) => {
                        op2 = land_exp_str;
                    }
                    Res::Temp(id) => {
                        op2 = format!("%{}", id);
                        s += &land_exp_str;
                    }
                    _ => unreachable!(),
                }
                s += &format!("\t%{0} = or {1}, {2}\n", info.temp_id, op1, op2);
                s += &format!("\t%{0} = ne %{1}, {2}\n", info.temp_id + 1, info.temp_id, 0);
                s += &format!("\tstore %{0}, @bool{1}\n", info.temp_id + 1, ans_val);
                info.temp_id += 2;

                s += &format!("\tjump %flag{0}\n", end_flag);
                s += &format!("%flag{0}:\n", true_flag);
                s += &format!("\tstore 1, @bool{0}\n", ans_val);
                s += &format!("\tjump %flag{0}\n", end_flag);
                s += &format!("%flag{0}:\n", end_flag);
                s += &format!("\t%{0} = load @bool{1}\n", info.temp_id, ans_val);
                res = Res::Temp(info.temp_id);
                info.temp_id += 1;
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
                let true_flag = info.flag_id;
                let false_flag = info.flag_id + 1;
                let end_flag = info.flag_id + 2;
                info.flag_id += 3;
                let (land_exp_str, land_exp_res) = land_exp.show(info);
                let (eq_exp_str, eq_exp_res) = eq_exp.show(info);
                let ans_val = info.var_id;
                s += &format!("\t@bool{0} = alloc i32\n", ans_val);
                info.var_id += 1;
                // 获取第一个操作数的字符串表示
                let mut op1 = "".to_string();
                match land_exp_res {
                    Res::Imm(imm) => {
                        op1 = land_exp_str;
                        s += &format!("\tbr {0}, %flag{1}, %flag{2}\n", op1, false_flag, true_flag);
                    }
                    Res::Temp(id) => {
                        op1 = format!("%{}", id);
                        s += &land_exp_str;
                        s += &format!("\tbr %{0}, %flag{1}, %flag{2}\n", id, false_flag, true_flag);
                    }
                    _ => unreachable!(),
                }
                s += &format!("%flag{0}:\n", false_flag);

                // 获取第二个操作数的字符串表示
                let mut op2 = "".to_string();
                match eq_exp_res {
                    Res::Imm(imm) => {
                        op2 = eq_exp_str;
                    }
                    Res::Temp(id) => {
                        op2 = format!("%{}", id);
                        s += &eq_exp_str;
                    }
                    _ => unreachable!(),
                }

                s += &format!("\t%{0} = ne {1}, {2}\n", info.temp_id, op1, 0);
                s += &format!("\t%{0} = ne {1}, {2}\n", info.temp_id + 1, op2, 0);
                s += &format!(
                    "\t%{0} = and %{1}, %{2}\n",
                    info.temp_id + 2,
                    info.temp_id + 1,
                    info.temp_id
                );
                s += &format!("\tstore %{0}, @bool{1}\n", info.temp_id + 2, ans_val);
                info.temp_id += 3;

                s += &format!("\tjump %flag{0}\n", end_flag);
                s += &format!("%flag{0}:\n", true_flag);
                s += &format!("\tstore 0, @bool{0}\n", ans_val);
                s += &format!("\tjump %flag{0}\n", end_flag);
                s += &format!("%flag{0}:\n", end_flag);
                s += &format!("\t%{0} = load @bool{1}\n", info.temp_id, ans_val);
                res = Res::Temp(info.temp_id);
                info.temp_id += 1;
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
                    Res::Imm(imm) => {
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
                    Res::Imm(imm) => {
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
                    Res::Imm(imm) => {
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
                    Res::Imm(imm) => {
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

trait GlobalShow {
    fn global_show(&self, info: &mut CompilerInfo) -> String;
}

impl GlobalShow for Decl {
    fn global_show(&self, info: &mut CompilerInfo) -> String {
        let mut s = "".to_string();
        match self {
            Decl::ConstDecl(const_decl) => {
                s += &const_decl.global_show(info);
            }
            Decl::VarDecl(var_decl) => {
                s += &var_decl.global_show(info);
            }
        }
        s
    }
}

impl GlobalShow for ConstDecl {
    fn global_show(&self, info: &mut CompilerInfo) -> String {
        let mut s = "".to_string();
        for const_def in &self.const_defs {
            s += &const_def.global_show(info);
        }
        s
    }
}

impl GlobalShow for ConstDef {
    fn global_show(&self, info: &mut CompilerInfo) -> String {
        let mut ret_s = "".to_string();
        let mut calculate_info = info
            .clone()
            .vars_table
            .into_iter()
            .map(|(k, (v, _))| (k, v))
            .collect();
        // 判断该定义是常量定义还是数组定义
        if self.dims.len() == 0 {
            match info.vars_table.get_mut(&self.ident) {
                None => {
                    let value = self.const_init_val.calculate(&mut calculate_info);
                    info.vars_table.insert(
                        self.ident.clone(),
                        (Variable::ConstINT(value), info.field_depth),
                    );
                }
                Some(value) => {
                    panic!("define a global const variable more than once!");
                }
            }
        } else {
            match info.vars_table.get_mut(&self.ident) {
                None => {
                    let mut dims: Vec<i32> = Vec::new();
                    let mut array_str = "i32".to_string();
                    for dim in &self.dims {
                        dims.push(dim.calculate(&mut calculate_info));
                    }
                    for dim in dims.iter().rev() {
                        array_str = format!("[{0}, {1}]", array_str, dim);
                    }
                    let mut val_idx = 0;
                    let init_str = self
                        .const_init_val
                        .global_array_init(info, &dims, &mut val_idx);
                    ret_s += &format!(
                        "global @{0}_global = alloc {1}, {2}\n",
                        self.ident, array_str, init_str
                    );
                    let new_var = Variable::Array(format!("@{0}_global", self.ident));
                    info.vars_table.insert(self.ident.clone(), (new_var, 0));
                }
                Some(value) => {
                    panic!("define a global const variable more than once!");
                }
            }
        }
        ret_s
    }
}

impl GlobalShow for VarDecl {
    fn global_show(&self, info: &mut CompilerInfo) -> String {
        let mut s = "".to_string();
        for var_def in &self.var_defs {
            s += &var_def.global_show(info);
        }
        s
    }
}

impl GlobalShow for VarDef {
    fn global_show(&self, info: &mut CompilerInfo) -> String {
        let mut s = "".to_string();
        match self {
            VarDef::Decl((var_name, dims)) => {
                let mut calculate_info = info
                    .clone()
                    .vars_table
                    .into_iter()
                    .map(|(k, (v, _))| (k, v))
                    .collect();
                let mut dims_i32: Vec<i32> = Vec::new();
                let mut array_str = "i32".to_string();
                for dim in dims {
                    dims_i32.push(dim.calculate(&mut calculate_info));
                }
                for dim in dims_i32.iter().rev() {
                    array_str = format!("[{0}, {1}]", array_str, dim);
                }

                s += &format!(
                    "global @{0}_global = alloc {1}, zeroinit\n",
                    var_name, array_str
                );
                if dims.len() == 0 {
                    info.vars_table.insert(
                        var_name.clone(),
                        (Variable::INT(format!("@{0}_global", var_name)), 0),
                    );
                } else {
                    info.vars_table.insert(
                        var_name.clone(),
                        (Variable::Array(format!("@{0}_global", var_name)), 0),
                    );
                }
            }
            VarDef::Def((var_name, dims, init_val)) => {
                let mut calculate_info = info
                    .clone()
                    .vars_table
                    .into_iter()
                    .map(|(k, (v, _))| (k, v))
                    .collect();

                if dims.len() == 0 {
                    let mut init_str = "".to_string();
                    match init_val {
                        InitVal::Exp(exp) => {
                            init_str += &exp.calculate(&mut calculate_info).to_string();
                        }
                        InitVal::Array(array) => unreachable!(),
                    }
                    s += &format!("global @{0}_global = alloc i32, {1}\n", var_name, init_str);
                    info.vars_table.insert(
                        var_name.clone(),
                        (Variable::INT(format!("@{0}_global", var_name)), 0),
                    );
                } else {
                    let mut dims_i32: Vec<i32> = Vec::new();
                    let mut array_str = "i32".to_string();
                    for dim in dims {
                        dims_i32.push(dim.calculate(&mut calculate_info));
                    }
                    for dim in dims_i32.iter().rev() {
                        array_str = format!("[{0}, {1}]", array_str, dim);
                    }

                    match init_val {
                        InitVal::Exp(exp) => unreachable!(),
                        InitVal::Array(array) => {
                            let mut val_idx = 0;
                            let init_str =
                                &init_val.global_array_init(info, &dims_i32, &mut val_idx);
                            s += &format!(
                                "global @{0}_global = alloc {1}, {2}\n",
                                var_name, array_str, init_str
                            );
                            let new_var = Variable::Array(format!("@{0}_global", var_name));
                            info.vars_table.insert(var_name.clone(), (new_var, 0));
                        }
                    }
                }
            }
        }
        s
    }
}

trait ArrayInit {
    fn global_array_init(
        &self,
        info: &mut CompilerInfo,
        dims: &Vec<i32>,
        val_idx: &mut usize,
    ) -> String;
    fn local_array_init(
        &self,
        info: &mut CompilerInfo,
        dims: &Vec<i32>,
        val_idx: &mut usize,
        base_ptr: String,
    ) -> String;
}

impl ArrayInit for ConstInitVal {
    fn global_array_init(
        &self,
        info: &mut CompilerInfo,
        dims: &Vec<i32>,
        val_idx: &mut usize,
    ) -> String {
        let mut init_s = "".to_string();
        let mut calculate_info: HashMap<String, Variable> = info
            .clone()
            .vars_table
            .into_iter()
            .map(|(k, (v, _))| (k, v))
            .collect();
        if dims.len() <= 0 {
            panic!("Wrong array dimension length!!");
        }
        // 一维数组
        if dims.len() == 1 {
            init_s += "{";
            let mut flag = 0;
            match self {
                ConstInitVal::Exp(exp) => {
                    panic!("Try to assign an i32 to array!");
                }
                ConstInitVal::Array(array) => {
                    while (*val_idx + flag as usize) < array.len() {
                        if flag >= dims[0] {
                            break;
                        }
                        match &array[*val_idx + flag as usize] {
                            ConstInitVal::Exp(exp) => {
                                let init_val = exp.calculate(&mut calculate_info).to_string();
                                if flag > 0 {
                                    init_s += &format!(", {0}", init_val);
                                } else {
                                    init_s += &format!("{0}", init_val);
                                }
                                flag += 1;
                            }
                            ConstInitVal::Array(array) => {
                                panic!("Try to assign an array to i32!");
                            }
                        }
                    }
                    for i in 0..dims[0] - flag {
                        if flag > 0 {
                            init_s += ", 0";
                        } else {
                            init_s += "0";
                            flag += 1;
                        }
                    }
                    init_s += "}";
                }
            }
            *val_idx += flag as usize;
        }
        //多维数组
        else {
            init_s += "{";
            for i in 0..dims[0] {
                if i > 0 {
                    init_s += ", ";
                }
                match self {
                    ConstInitVal::Exp(exp) => {
                        panic!("Try to assign an i32 to array!");
                    }
                    ConstInitVal::Array(array) => {
                        if *val_idx >= array.len() {
                            init_s += &global_zero_array(&dims[1..].to_vec());
                            continue;
                        }
                        match &array[*val_idx] {
                            ConstInitVal::Exp(_exp) => {
                                init_s +=
                                    &self.global_array_init(info, &dims[1..].to_vec(), val_idx);
                            }
                            ConstInitVal::Array(_array) => {
                                let mut new_idx = 0;
                                init_s += &array[*val_idx].global_array_init(
                                    info,
                                    &dims[1..].to_vec(),
                                    &mut new_idx,
                                );
                                *val_idx += 1;
                            }
                        }
                    }
                }
            }
            init_s += "}";
        }
        init_s
    }

    fn local_array_init(
        &self,
        info: &mut CompilerInfo,
        dims: &Vec<i32>,
        val_idx: &mut usize,
        base_ptr: String,
    ) -> String {
        let mut init_s = "".to_string();
        let mut calculate_info: HashMap<String, Variable> = info
            .clone()
            .vars_table
            .into_iter()
            .map(|(k, (v, _))| (k, v))
            .collect();
        if dims.len() <= 0 {
            panic!("Wrong array dimension length!!");
        }
        // 一维数组
        if dims.len() == 1 {
            let mut flag = 0;
            match self {
                ConstInitVal::Exp(exp) => {
                    panic!("Try to assign an i32 to array!");
                }
                ConstInitVal::Array(array) => {
                    while (*val_idx + flag as usize) < array.len() {
                        if flag >= dims[0] {
                            break;
                        }
                        match &array[*val_idx + flag as usize] {
                            ConstInitVal::Exp(exp) => {
                                let init_val = exp.calculate(&mut calculate_info).to_string();
                                init_s += &format!(
                                    "\t%{0} = getelemptr {1}, {2}\n\tstore {3}, %{0}\n",
                                    info.temp_id, base_ptr, flag, init_val
                                );
                                info.temp_id += 1;
                                flag += 1;
                            }
                            ConstInitVal::Array(array) => {
                                panic!("Try to assign an array to i32!");
                            }
                        }
                    }
                    for i in 0..dims[0] - flag {
                        init_s += &format!(
                            "\t%{0} = getelemptr {1}, {2}\n\tstore 0, %{0}\n",
                            info.temp_id, base_ptr, flag
                        );
                        info.temp_id += 1;
                        flag += 1;
                    }
                }
            }
            *val_idx += flag as usize;
        }
        //多维数组
        else {
            for i in 0..dims[0] {
                match self {
                    ConstInitVal::Exp(exp) => {
                        panic!("Try to assign an i32 to array!");
                    }
                    ConstInitVal::Array(array) => {
                        if *val_idx >= array.len() {
                            init_s += &format!(
                                "\t%{0} = getelemptr {1}, {2}\n",
                                info.temp_id, base_ptr, i
                            );
                            let sub_base = format!("%{0}", info.temp_id);
                            info.temp_id += 1;
                            init_s += &local_zero_array(info, &dims[1..].to_vec(), sub_base);
                            continue;
                        }
                        match &array[*val_idx] {
                            ConstInitVal::Exp(_exp) => {
                                init_s += &format!(
                                    "\t%{0} = getelemptr {1}, {2}\n",
                                    info.temp_id, base_ptr, i
                                );
                                let sub_base = format!("%{0}", info.temp_id);
                                info.temp_id += 1;
                                init_s += &self.local_array_init(
                                    info,
                                    &dims[1..].to_vec(),
                                    val_idx,
                                    sub_base,
                                );
                            }
                            ConstInitVal::Array(_array) => {
                                let mut new_idx = 0;
                                init_s += &format!(
                                    "\t%{0} = getelemptr {1}, {2}\n",
                                    info.temp_id, base_ptr, i
                                );
                                let sub_base = format!("%{0}", info.temp_id);
                                info.temp_id += 1;
                                init_s += &array[*val_idx].local_array_init(
                                    info,
                                    &dims[1..].to_vec(),
                                    &mut new_idx,
                                    sub_base,
                                );
                                *val_idx += 1;
                            }
                        }
                    }
                }
            }
        }
        init_s
    }
}

impl ArrayInit for InitVal {
    fn global_array_init(
        &self,
        info: &mut CompilerInfo,
        dims: &Vec<i32>,
        val_idx: &mut usize,
    ) -> String {
        let mut init_s = "".to_string();
        let mut calculate_info: HashMap<String, Variable> = info
            .clone()
            .vars_table
            .into_iter()
            .map(|(k, (v, _))| (k, v))
            .collect();
        if dims.len() <= 0 {
            panic!("Wrong array dimension length!!");
        }
        // 一维数组
        if dims.len() == 1 {
            init_s += "{";
            let mut flag = 0;
            match self {
                InitVal::Exp(exp) => {
                    panic!("Try to assign an i32 to array!");
                }
                InitVal::Array(array) => {
                    while (*val_idx + flag as usize) < array.len() {
                        if flag >= dims[0] {
                            break;
                        }
                        match &array[*val_idx + flag as usize] {
                            InitVal::Exp(exp) => {
                                let init_val = exp.calculate(&mut calculate_info).to_string();
                                if flag > 0 {
                                    init_s += &format!(", {0}", init_val);
                                } else {
                                    init_s += &format!("{0}", init_val);
                                }
                                flag += 1;
                            }
                            InitVal::Array(array) => {
                                panic!("Try to assign an array to i32!");
                            }
                        }
                    }
                    for i in 0..dims[0] - flag {
                        if flag > 0 {
                            init_s += ", 0";
                        } else {
                            init_s += "0";
                            flag += 1;
                        }
                    }
                    init_s += "}";
                }
            }
            *val_idx += flag as usize;
        }
        //多维数组
        else {
            init_s += "{";
            for i in 0..dims[0] {
                if i > 0 {
                    init_s += ", ";
                }
                match self {
                    InitVal::Exp(exp) => {
                        panic!("Try to assign an i32 to array!");
                    }
                    InitVal::Array(array) => {
                        if *val_idx >= array.len() {
                            init_s += &global_zero_array(&dims[1..].to_vec());
                            continue;
                        }
                        match &array[*val_idx] {
                            InitVal::Exp(_exp) => {
                                init_s +=
                                    &self.global_array_init(info, &dims[1..].to_vec(), val_idx);
                            }
                            InitVal::Array(_array) => {
                                let mut new_idx = 0;
                                init_s += &array[*val_idx].global_array_init(
                                    info,
                                    &dims[1..].to_vec(),
                                    &mut new_idx,
                                );
                                *val_idx += 1;
                            }
                        }
                    }
                }
            }
            init_s += "}";
        }
        init_s
    }

    fn local_array_init(
        &self,
        info: &mut CompilerInfo,
        dims: &Vec<i32>,
        val_idx: &mut usize,
        base_ptr: String,
    ) -> String {
        let mut init_s = "".to_string();
        if dims.len() <= 0 {
            panic!("Wrong array dimension length!!");
        }
        // 一维数组
        if dims.len() == 1 {
            let mut flag = 0;
            match self {
                InitVal::Exp(exp) => {
                    panic!("Try to assign an i32 to array!");
                }
                InitVal::Array(array) => {
                    while (*val_idx + flag as usize) < array.len() {
                        if flag >= dims[0] {
                            break;
                        }
                        match &array[*val_idx + flag as usize] {
                            InitVal::Exp(exp) => {
                                let (init_val, init_res) = exp.show(info);
                                match init_res {
                                    Res::Imm(imm) => {
                                        init_s += &format!(
                                            "\t%{0} = getelemptr {1}, {2}\n",
                                            info.temp_id, base_ptr, flag
                                        );
                                        init_s +=
                                            &format!("\tstore {0}, %{1}\n", imm, info.temp_id);
                                        info.temp_id += 1;
                                    }
                                    Res::Temp(temp) => {
                                        init_s += &init_val;
                                        init_s += &format!(
                                            "\t%{0} = getelemptr {1}, {2}\n",
                                            info.temp_id, base_ptr, flag
                                        );
                                        init_s +=
                                            &format!("\tstore %{0}, %{1}\n", temp, info.temp_id);
                                        info.temp_id += 1;
                                    }
                                    _ => unreachable!(),
                                }
                                flag += 1;
                            }
                            InitVal::Array(array) => {
                                panic!("Try to assign an array to i32!");
                            }
                        }
                    }
                    for i in 0..dims[0] - flag {
                        init_s += &format!(
                            "\t%{0} = getelemptr {1}, {2}\n",
                            info.temp_id, base_ptr, flag
                        );
                        init_s += &format!("\tstore 0, %{0}\n", info.temp_id);
                        info.temp_id += 1;
                        flag += 1;
                    }
                }
            }
            *val_idx += flag as usize;
        }
        //多维数组
        else {
            for i in 0..dims[0] {
                match self {
                    InitVal::Exp(exp) => {
                        panic!("Try to assign an i32 to array!");
                    }
                    InitVal::Array(array) => {
                        if *val_idx >= array.len() {
                            init_s += &format!(
                                "\t%{0} = getelemptr {1}, {2}\n",
                                info.temp_id, base_ptr, i
                            );
                            let sub_base = format!("%{0}", info.temp_id);
                            info.temp_id += 1;
                            init_s += &local_zero_array(info, &dims[1..].to_vec(), sub_base);
                            continue;
                        }
                        match &array[*val_idx] {
                            InitVal::Exp(_exp) => {
                                init_s += &format!(
                                    "\t%{0} = getelemptr {1}, {2}\n",
                                    info.temp_id, base_ptr, i
                                );
                                let sub_base = format!("%{0}", info.temp_id);
                                info.temp_id += 1;
                                init_s += &self.local_array_init(
                                    info,
                                    &dims[1..].to_vec(),
                                    val_idx,
                                    sub_base,
                                );
                            }
                            InitVal::Array(_array) => {
                                init_s += &format!(
                                    "\t%{0} = getelemptr {1}, {2}\n",
                                    info.temp_id, base_ptr, i
                                );
                                let sub_base = format!("%{0}", info.temp_id);
                                info.temp_id += 1;

                                let mut new_idx = 0;
                                init_s += &array[*val_idx].local_array_init(
                                    info,
                                    &dims[1..].to_vec(),
                                    &mut new_idx,
                                    sub_base,
                                );
                                *val_idx += 1;
                            }
                        }
                    }
                }
            }
        }
        init_s
    }
}
