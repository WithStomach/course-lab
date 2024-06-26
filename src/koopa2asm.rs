use koopa::ir::{dfg::DataFlowGraph, entities::Value, BinaryOp, Program, ValueKind};
use std::collections::HashMap;

enum Res {
    Nothing,
    Imm,
    Register(i32),
    Return(i32),
}

fn get_register_name(register_id: &i32) -> String {
    if *register_id >= 0 {
        format!("{0}(sp)", register_id * 4)
    } else {
        format!("global_var_{0}", -register_id)
    }
}

trait GenerateAsm {
    fn generate(
        &self,
        dfg: Option<&DataFlowGraph>,
        register_id: &mut i32,
        value_reg_map: &mut HashMap<Value, i32>,
        source_prog: &koopa::ir::Program,
    ) -> (String, Res);
}

impl GenerateAsm for koopa::ir::Program {
    fn generate(
        &self,
        _dfg: Option<&DataFlowGraph>,
        register_id: &mut i32,
        value_reg_map: &mut HashMap<Value, i32>,
        source_prog: &koopa::ir::Program,
    ) -> (String, Res) {
        let mut s = "".to_string();
        let mut global_id = 1;
        for &value in self.inst_layout() {
            let data = self.borrow_value(value);
            s += &format!(
                "\t.data\n\t.globl global_var_{0}\nglobal_var_{0}:\n",
                global_id,
            );
            s += &data
                .generate(None, register_id, value_reg_map, source_prog)
                .0;
            value_reg_map.insert(value, -global_id);
            global_id += 1;
        }
        for &func in self.func_layout() {
            let func_data = self.func(func);
            if func_data.layout().entry_bb() == None {
                continue;
            }
            s += &func_data
                .generate(
                    None,
                    &mut register_id.clone(),
                    &mut value_reg_map.clone(),
                    source_prog,
                )
                .0;
        }
        (s, Res::Nothing)
    }
}

impl GenerateAsm for koopa::ir::FunctionData {
    fn generate(
        &self,
        _dfg: Option<&DataFlowGraph>,
        register_id: &mut i32,
        value_reg_map: &mut HashMap<Value, i32>,
        source_prog: &koopa::ir::Program,
    ) -> (String, Res) {
        let mut pre_str = "".to_string();
        pre_str += "\n\t.text\n";
        pre_str += "\t.global ";
        pre_str += &self.name()[1..];
        pre_str += "\n";
        pre_str += &self.name()[1..];
        pre_str += ":\n";
        let mut s = "".to_string();
        let mut flag = 0;
        for (&bb, node) in self.layout().bbs() {
            flag += 1;
            let bb_data = self.dfg().bb(bb);
            match bb_data.name() {
                Some(name) => {
                    s += &name[1..];
                    s += ":\n";
                }
                None => {}
            }
            for &inst in node.insts().keys() {
                let value_data = self.dfg().value(inst);
                let (ret_str, ret_res) =
                    value_data.generate(Some(self.dfg()), register_id, value_reg_map, source_prog);
                s += &ret_str;
                match ret_res {
                    Res::Nothing => {}
                    Res::Imm => {}
                    Res::Register(idx) => {
                        value_reg_map.insert(inst, idx);
                    }
                    Res::Return(ret) => s += &format!("\tj {0}_end\n", &self.name()[1..]),
                }
            }
        }
        let mut stack_len = *register_id * 4;
        if stack_len % 16 != 0 {
            stack_len += 16 - stack_len % 16;
        }
        if flag != 0 {
            pre_str += &format!("\tli t5, {0}\n\tadd sp, sp, t5\n", -stack_len);
        }
        let end_str = format!(
            "{0}_end:\n\tli t5, {1}\n\tadd sp, sp, t5\n\tret\n",
            &self.name()[1..],
            stack_len
        );
        let ans_s = pre_str + &s + &end_str;
        (ans_s, Res::Nothing)
    }
}

impl GenerateAsm for koopa::ir::entities::ValueData {
    fn generate(
        &self,
        dfg: Option<&DataFlowGraph>,
        register_id: &mut i32,
        value_reg_map: &mut HashMap<Value, i32>,
        source_prog: &koopa::ir::Program,
    ) -> (String, Res) {
        let mut s = "".to_string();
        let mut res = Res::Nothing;
        match self.kind() {
            ValueKind::Integer(int) => {
                s += &int.value().to_string();
                res = Res::Imm;
            }
            ValueKind::Return(ret) => {
                let dfg_used = dfg.unwrap();
                match ret.value() {
                    None => {}
                    Some(ret_val) => {
                        let ret_value = dfg_used.value(ret_val);
                        match value_reg_map.get(&ret.value().unwrap()) {
                            None => {
                                let (ret_str, ret_res) = ret_value.generate(
                                    dfg,
                                    register_id,
                                    value_reg_map,
                                    source_prog,
                                );
                                match ret_res {
                                    Res::Imm => {
                                        s += &format!("\tli a0, {0}\n", ret_str);
                                    }
                                    Res::Register(idx) => {
                                        s += &format!("\tlw a0, {0}\n", get_register_name(&idx));
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            Some(idx) => {
                                s += &format!("\tlw a0, {0}\n", get_register_name(&idx));
                            }
                        }
                    }
                }
                res = Res::Return(0);
            }
            ValueKind::Binary(exp) => {
                let dfg_used = dfg.unwrap();
                let op = exp.op();
                let lhs = dfg_used.value(exp.lhs());
                let rhs = dfg_used.value(exp.rhs());
                // 左右操作数以及最终结果存放在哪些寄存器内
                let mut lhs_reg = -1;
                let mut rhs_reg = -1;
                let mut res_reg = -1;

                match value_reg_map.get(&exp.lhs()) {
                    None => {
                        let (lhs_str, lhs_res) =
                            lhs.generate(dfg, register_id, value_reg_map, source_prog);
                        match lhs_res {
                            Res::Nothing => {}
                            Res::Imm => {
                                s += &format!(
                                    "\tli t5, {0}\n\tsw t5, {1}\n",
                                    lhs_str,
                                    get_register_name(register_id),
                                );
                                lhs_reg = *register_id;
                                *register_id += 1;
                            }
                            Res::Register(id) => {
                                // 若左操作数的结果已经存入第id个临时寄存器中，直接使用即可
                                s += &lhs_str;
                                lhs_reg = id;
                            }
                            _ => {}
                        }
                    }
                    Some(idx) => {
                        lhs_reg = *idx;
                    }
                }
                match value_reg_map.get(&exp.rhs()) {
                    None => {
                        let (rhs_str, rhs_res) =
                            rhs.generate(dfg, register_id, value_reg_map, source_prog);
                        // 对右操作数，处理过程与左操作数相同
                        match rhs_res {
                            Res::Nothing => {}
                            Res::Imm => {
                                s += &format!(
                                    "\tli t5, {0}\n\tsw t5, {1}\n",
                                    rhs_str,
                                    get_register_name(register_id),
                                );
                                rhs_reg = *register_id;
                                *register_id += 1;
                            }
                            Res::Register(id) => {
                                s += &rhs_str;
                                rhs_reg = id;
                            }
                            _ => {}
                        }
                    }
                    Some(idx) => {
                        rhs_reg = *idx;
                    }
                }

                res_reg = *register_id;
                *register_id += 1;

                // 找出对应操作
                match op {
                    BinaryOp::Add => {
                        s += &format!(
                            "\tlw t5, {0}\n\tlw t6, {1}\n\tadd t5, t5, t6\n\tsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Sub => {
                        s += &format!(
                            "\tlw t5, {0}\n\tlw t6, {1}\n\tsub t5, t5, t6\n\tsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Mul => {
                        s += &format!(
                            "\tlw t5, {0}\n\tlw t6, {1}\n\tmul t5, t5, t6\n\tsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Div => {
                        s += &format!(
                            "\tlw t5, {0}\n\tlw t6, {1}\n\tdiv t5, t5, t6\n\tsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Mod => {
                        s += &format!(
                            "\tlw t5, {0}\n\tlw t6, {1}\n\trem t5, t5, t6\n\tsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::And => {
                        s += &format!(
                            "\tlw t5, {0}\n\tlw t6, {1}\n\tand t5, t5, t6\n\tsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Or => {
                        s += &format!(
                            "\tlw t5, {0}\n\tlw t6, {1}\n\tor t5, t5, t6\n\tsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Eq => {
                        // a == b <==> (a xor b) == 0
                        s += &format!(
                            "\tlw t5, {0}\n\tlw t6, {1}\n\txor t5, t5, t6\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                        );
                        s += &format!("\tseqz t5, t5\n\tsw t5, {0}\n", get_register_name(&res_reg));
                    }
                    BinaryOp::NotEq => {
                        // a == b <==> (a xor b) == 0
                        s += &format!(
                            "\tlw t5, {0}\n\tlw t6, {1}\n\txor t5, t5, t6\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                        );
                        s += &format!("\tsnez t5, t5\n\tsw t5, {0}\n", get_register_name(&res_reg));
                    }
                    BinaryOp::Lt => {
                        s += &format!(
                            "\tlw t5, {0}\n\tlw t6, {1}\n\tslt t5, t5, t6\n\tsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Gt => {
                        s += &format!(
                            "\tlw t5, {0}\n\tlw t6, {1}\n\tsgt t5, t5, t6\n\tsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Le => {
                        // a <= b <==> a - b <= 0
                        // 首先判断是否有 a < b
                        s += &format!(
                            "\tlw t5, {0}\n\tlw t6, {1}\n\tslt t4, t5, t6\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                        );
                        // 再判断是否 a == b
                        s += &format!("\txor t3, t5, t6\n\tseqz t3, t3\n");
                        // 将两个判断结果作或
                        s += &format!(
                            "\tor t5, t4, t3\n\tsw t5, {0}\n",
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Ge => {
                        s += &format!(
                            "\tlw t5, {0}\n\tlw t6, {1}\n\tsgt t4, t5, t6\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                        );
                        // 再判断是否 a == b
                        s += &format!("\txor t3, t5, t6\n\tseqz t3, t3\n");
                        // 将两个判断结果作或
                        s += &format!(
                            "\tor t5, t4, t3\n\tsw t5, {0}\n",
                            get_register_name(&res_reg),
                        );
                    }
                    _ => panic!("4"),
                }
                res = Res::Register(res_reg);
            }
            ValueKind::Alloc(alloc) => {
                println!("{:?}", alloc);
                res = Res::Register(*register_id);
                *register_id += 1;
            }
            ValueKind::Load(load) => {
                match value_reg_map.get(&load.src()) {
                    None => panic!("3"),
                    Some(i) => {
                        s += &format!(
                            "\tlw t5, {0}\n\tsw t5, {1}\n",
                            get_register_name(&i),
                            get_register_name(register_id)
                        );
                    }
                }
                res = Res::Register(*register_id);
                *register_id += 1;
            }
            ValueKind::Store(store) => {
                let dfg_used = dfg.unwrap();
                let value = dfg_used.value(store.value());
                match value_reg_map.get(&store.value()) {
                    None => {
                        let (value_str, value_res) =
                            value.generate(dfg, register_id, value_reg_map, source_prog);
                        match value_res {
                            Res::Nothing => {}
                            Res::Imm => {
                                s += &format!("\tli t5, {0}\n", value_str);
                            }
                            Res::Register(id) => {
                                s += &format!("\tlw t5, {0}\n", get_register_name(&id));
                            }
                            _ => {}
                        }
                    }
                    Some(i) => {
                        s += &format!("\tlw t5, {0}\n", get_register_name(&i));
                    }
                }
                match value_reg_map.get(&store.dest()) {
                    Some(i) => {
                        s += &format!("\tsw t5, {0}\n", get_register_name(i));
                    }
                    None => panic!("1"),
                }
            }
            ValueKind::Jump(jump) => {
                let dfg_used = dfg.unwrap();
                let target_bb = dfg_used.bb(jump.target());
                match target_bb.name() {
                    Some(name) => {
                        s += &format!("\tj {0}\n", &name[1..]);
                    }
                    None => unreachable!(),
                }
            }
            ValueKind::Branch(branch) => {
                let dfg_used = dfg.unwrap();
                match value_reg_map.get(&branch.cond()) {
                    Some(i) => {
                        s += &format!("\tlw t5, {0}\n\tbnez t5, ", get_register_name(i));
                        let true_bb = dfg_used.bb(branch.true_bb());
                        let false_bb = dfg_used.bb(branch.false_bb());
                        match true_bb.name() {
                            Some(name) => {
                                s += &name[1..];
                                s += "\n";
                            }
                            _ => unreachable!(),
                        }
                        s += "\tj ";
                        match false_bb.name() {
                            Some(name) => {
                                s += &name[1..];
                                s += "\n";
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            ValueKind::FuncArgRef(arg_val) => {
                println!("{:?}", arg_val)
            }
            ValueKind::GlobalAlloc(globl_alloc) => {
                let init_val = source_prog.borrow_value(globl_alloc.init());
                s += "\t.word ";
                s += &init_val
                    .generate(dfg, register_id, value_reg_map, source_prog)
                    .0;
                s += "\n";
            }
            ValueKind::ZeroInit(zero_init) => {
                s += "0";
            }
            _ => panic!("2"),
        }
        (s, res)
    }
}

pub fn koopa2asm(program: &Program) -> String {
    let mut register_recorder = 0;
    let mut value_reg_map: HashMap<Value, i32> = HashMap::new();
    program
        .generate(None, &mut register_recorder, &mut value_reg_map, program)
        .0
}
