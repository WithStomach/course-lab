use koopa::ir::{dfg::DataFlowGraph, entities::Value, BinaryOp, Program, ValueKind};
use std::collections::HashMap;

enum Res {
    Nothing,
    Imm,
    Register(i32),
    Return(i32),
}

fn get_register_name(register_id: &i32) -> String {
    // if *register_id <= 6 {
    //     format!("t{0}", register_id)
    // } else {
    //     format!("a{0}", register_id - 7)
    // }
    format!("{0}(sp)", register_id * 4)
}

trait GenerateAsm {
    fn generate(
        &self,
        dfg: Option<&DataFlowGraph>,
        register_id: &mut i32,
        value_reg_map: &mut HashMap<Value, i32>,
    ) -> (String, Res);
}

impl GenerateAsm for koopa::ir::Program {
    fn generate(
        &self,
        _dfg: Option<&DataFlowGraph>,
        register_id: &mut i32,
        value_reg_map: &mut HashMap<Value, i32>,
    ) -> (String, Res) {
        let mut s = "".to_string();
        s += ".text\n";
        for &func in self.func_layout() {
            let func_data = self.func(func);
            s += ".global ";
            s += &func_data.name()[1..];
            s += "\n";
        }
        for &func in self.func_layout() {
            let func_data = self.func(func);
            s += &func_data.generate(None, register_id, value_reg_map).0;
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
    ) -> (String, Res) {
        let mut pre_str = "".to_string();
        pre_str += &self.name()[1..];
        pre_str += ":\n";
        let mut s = "".to_string();
        for (&_bb, node) in self.layout().bbs() {
            for &inst in node.insts().keys() {
                let value_data = self.dfg().value(inst);
                let (ret_str, ret_res) =
                    value_data.generate(Some(self.dfg()), register_id, value_reg_map);
                s += &ret_str;
                match ret_res {
                    Res::Nothing => {}
                    Res::Imm => {}
                    Res::Register(idx) => {
                        value_reg_map.insert(inst, idx);
                    }
                    _ => {}
                }
            }
        }
        let mut stack_len = *register_id * 4;
        if stack_len % 16 != 0 {
            stack_len += 16 - stack_len % 16;
        }
        pre_str += &format!("addi sp, sp, -{0}\n", stack_len);
        let end_str = format!("addi sp, sp, {0}\nret\n", stack_len);
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
    ) -> (String, Res) {
        let mut s = "".to_string();
        let dfg_used = dfg.unwrap();
        let mut res = Res::Nothing;
        match self.kind() {
            ValueKind::Integer(int) => {
                s += &int.value().to_string();
                res = Res::Imm;
            }
            ValueKind::Return(ret) => {
                let ret_value = dfg_used.value(ret.value().unwrap());
                match value_reg_map.get(&ret.value().unwrap()) {
                    None => {
                        let (ret_str, ret_res) =
                            ret_value.generate(dfg, register_id, value_reg_map);
                        match ret_res {
                            Res::Nothing => {}
                            Res::Imm => {
                                s += &format!("li a0, {0}\n", ret_str);
                            }
                            Res::Register(idx) => {
                                s += &format!("lw a0, {0}\n", get_register_name(&idx));
                            }
                            _ => {}
                        }
                    }
                    Some(idx) => {
                        s += &format!("lw a0, {0}\n", get_register_name(&idx));
                    }
                }
                res = Res::Return(0);
            }
            ValueKind::Binary(exp) => {
                let op = exp.op();
                let lhs = dfg_used.value(exp.lhs());
                let rhs = dfg_used.value(exp.rhs());
                // 左右操作数以及最终结果存放在哪些寄存器内
                let mut lhs_reg = -1;
                let mut rhs_reg = -1;
                let mut res_reg = -1;

                match value_reg_map.get(&exp.lhs()) {
                    None => {
                        let (lhs_str, lhs_res) = lhs.generate(dfg, register_id, value_reg_map);
                        match lhs_res {
                            Res::Nothing => {}
                            Res::Imm => {
                                s += &format!(
                                    "li t5, {0}\nsw t5, {1}\n",
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
                        let (rhs_str, rhs_res) = rhs.generate(dfg, register_id, value_reg_map);
                        // 对右操作数，处理过程与左操作数相同
                        match rhs_res {
                            Res::Nothing => {}
                            Res::Imm => {
                                s += &format!(
                                    "li t5, {0}\nsw t5, {1}\n",
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
                            "lw t5, {0}\nlw t6, {1}\nadd t5, t5, t6\nsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Sub => {
                        s += &format!(
                            "lw t5, {0}\nlw t6, {1}\nsub t5, t5, t6\nsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Mul => {
                        s += &format!(
                            "lw t5, {0}\nlw t6, {1}\nmul t5, t5, t6\nsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Div => {
                        s += &format!(
                            "lw t5, {0}\nlw t6, {1}\ndiv t5, t5, t6\nsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Mod => {
                        s += &format!(
                            "lw t5, {0}\nlw t6, {1}\nrem t5, t5, t6\nsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::And => {
                        s += &format!(
                            "lw t5, {0}\nlw t6, {1}\nand t5, t5, t6\nsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Or => {
                        s += &format!(
                            "lw t5, {0}\nlw t6, {1}\nor t5, t5, t6\nsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Eq => {
                        // a == b <==> (a xor b) == 0
                        s += &format!(
                            "lw t5, {0}\nlw t6, {1}\nxor t5, t5, t6\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                        );
                        s += &format!("seqz t5, t5\nsw t5, {0}\n", get_register_name(&res_reg));
                    }
                    BinaryOp::NotEq => {
                        // a == b <==> (a xor b) == 0
                        s += &format!(
                            "lw t5, {0}\nlw t6, {1}\nxor t5, t5, t6\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                        );
                        s += &format!("snez t5, t5\nsw t5, {0}\n", get_register_name(&res_reg));
                    }
                    BinaryOp::Lt => {
                        s += &format!(
                            "lw t5, {0}\nlw t6, {1}\nslt t5, t5, t6\nsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Gt => {
                        s += &format!(
                            "lw t5, {0}\nlw t6, {1}\nsgt t5, t5, t6\nsw t5, {2}\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                            get_register_name(&res_reg),
                        );
                    }
                    BinaryOp::Le => {
                        // a <= b <==> a - b <= 0
                        // 首先判断是否有 a < b
                        s += &format!(
                            "lw t5, {0}\nlw t6, {1}\nslt t4, t5, t6\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                        );
                        // 再判断是否 a == b
                        s += &format!("xor t3, t5, t6\nseqz t3, t3\n");
                        // 将两个判断结果作或
                        s += &format!("or t5, t4, t3\nsw t5, {0}\n", get_register_name(&res_reg),);
                    }
                    BinaryOp::Ge => {
                        s += &format!(
                            "lw t5, {0}\nlw t6, {1}\nsgt t4, t5, t6\n",
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg),
                        );
                        // 再判断是否 a == b
                        s += &format!("xor t3, t5, t6\nseqz t3, t3\n");
                        // 将两个判断结果作或
                        s += &format!("or t5, t4, t3\nsw t5, {0}\n", get_register_name(&res_reg),);
                    }
                    _ => unreachable!(),
                }
                res = Res::Register(res_reg);
            }
            _ => unreachable!(),
        }
        (s, res)
    }
}

pub fn koopa2asm(program: &Program) -> String {
    let mut register_recorder = 0;
    let mut value_reg_map: HashMap<Value, i32> = HashMap::new();
    program
        .generate(None, &mut register_recorder, &mut value_reg_map)
        .0
}
