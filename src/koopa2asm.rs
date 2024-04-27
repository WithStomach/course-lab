use koopa::ir::{dfg::DataFlowGraph, entities::Value, BinaryOp, Program, ValueKind};
use std::collections::HashMap;

enum Res {
    Nothing,
    Imm,
    Register(i32),
    Return,
}

fn get_register_name(register_id: &i32) -> String {
    if *register_id <= 6 {
        format!("t{0}", register_id)
    } else {
        format!("a{0}", register_id - 7)
    }
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
        s += "    .text\n";
        for &func in self.func_layout() {
            let func_data = self.func(func);
            s += "    .global ";
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
        let mut s = "".to_string();
        s += &self.name()[1..];
        s += ":\n";
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
                    Res::Return => {
                        println!("!!!");
                        break;
                    }
                }
            }
        }
        (s, Res::Nothing)
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
                                s += &format!("    li a0, {0}\n", ret_str);
                            }
                            Res::Register(idx) => {
                                s += &format!("    mv a0, {0}\n", get_register_name(&idx));
                            }
                            _ => {}
                        }
                    }
                    Some(idx) => {
                        s += &format!("    mv a0, {0}\n", get_register_name(&idx));
                    }
                }

                s += "    ret\n";
                res = Res::Return;
            }
            ValueKind::Binary(exp) => {
                let op = exp.op();
                let lhs = dfg_used.value(exp.lhs());
                let rhs = dfg_used.value(exp.rhs());
                // 左右操作数以及最终结果存放在哪些寄存器内
                let mut lhs_reg = -1;
                let mut rhs_reg = -1;
                let mut res_reg = -1;
                let mut lhs_imm = false;
                let mut rhs_imm = false;

                match value_reg_map.get(&exp.lhs()) {
                    None => {
                        let (lhs_str, lhs_res) = lhs.generate(dfg, register_id, value_reg_map);
                        match lhs_res {
                            Res::Nothing => {}
                            Res::Imm => {
                                // 对于立即数，将其放入一个新的临时寄存器中, 且最终结果也可以放入该寄存器中
                                s += &format!(
                                    "    li {0}, {1}\n",
                                    get_register_name(register_id),
                                    lhs_str
                                );
                                lhs_reg = *register_id;
                                res_reg = *register_id;
                                *register_id += 1;
                                lhs_imm = true;
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
                                    "    li {0}, {1}\n",
                                    get_register_name(register_id),
                                    rhs_str
                                );
                                rhs_reg = *register_id;
                                if res_reg < 0 {
                                    res_reg = *register_id;
                                }
                                *register_id += 1;
                                rhs_imm = true;
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

                if res_reg < 0 {
                    res_reg = *register_id;
                    *register_id += 1;
                }

                // 若左右都存放了立即数，计算完成后存放右侧立即数的寄存器可以释放
                if lhs_imm && rhs_imm {
                    *register_id -= 1;
                }

                // 找出对应操作
                match op {
                    BinaryOp::Add => {
                        s += &format!(
                            "    add {0}, {1}, {2}\n",
                            get_register_name(&res_reg),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                    }
                    BinaryOp::Sub => {
                        s += &format!(
                            "    sub {0}, {1}, {2}\n",
                            get_register_name(&res_reg),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                    }
                    BinaryOp::Mul => {
                        s += &format!(
                            "    mul {0}, {1}, {2}\n",
                            get_register_name(&res_reg),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                    }
                    BinaryOp::Div => {
                        s += &format!(
                            "    div {0}, {1}, {2}\n",
                            get_register_name(&res_reg),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                    }
                    BinaryOp::Mod => {
                        s += &format!(
                            "    rem {0}, {1}, {2}\n",
                            get_register_name(&res_reg),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                    }
                    BinaryOp::And => {
                        s += &format!(
                            "    and {0}, {1}, {2}\n",
                            get_register_name(&res_reg),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                    }
                    BinaryOp::Or => {
                        s += &format!(
                            "    or {0}, {1}, {2}\n",
                            get_register_name(&res_reg),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                    }
                    BinaryOp::Eq => {
                        // a == b <==> (a xor b) == 0
                        s += &format!(
                            "    xor {0}, {1}, {2}\n",
                            get_register_name(&res_reg),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                        s += &format!("    seqz {0}, {0}\n", get_register_name(&res_reg));
                    }
                    BinaryOp::NotEq => {
                        // a == b <==> (a xor b) == 0
                        s += &format!(
                            "    xor {0}, {1}, {2}\n",
                            get_register_name(&res_reg),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                        s += &format!("    snez {0}, {0}\n", get_register_name(&res_reg));
                    }
                    BinaryOp::Lt => {
                        s += &format!(
                            "    slt {0}, {1}, {2}\n",
                            get_register_name(&res_reg),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                    }
                    BinaryOp::Gt => {
                        s += &format!(
                            "    sgt {0}, {1}, {2}\n",
                            get_register_name(&res_reg),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                    }
                    BinaryOp::Le => {
                        // a <= b <==> a - b <= 0
                        // 首先判断是否有 a < b
                        s += &format!(
                            "    slt {0}, {1}, {2}\n",
                            get_register_name(&res_reg),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                        // 再判断是否 a == b
                        s += &format!(
                            "    xor {0}, {1}, {2}\n",
                            get_register_name(register_id),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                        s += &format!("    seqz {0}, {0}\n", get_register_name(register_id));
                        // 将两个判断结果作或
                        s += &format!(
                            "    or {0}, {0}, {1}\n",
                            get_register_name(&res_reg),
                            get_register_name(register_id)
                        );
                    }
                    BinaryOp::Ge => {
                        s += &format!(
                            "    sgt {0}, {1}, {2}\n",
                            get_register_name(&res_reg),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                        s += &format!(
                            "    xor {0}, {1}, {2}\n",
                            get_register_name(register_id),
                            get_register_name(&lhs_reg),
                            get_register_name(&rhs_reg)
                        );
                        s += &format!("    seqz {0}, {0}\n", get_register_name(register_id));
                        s += &format!(
                            "    or {0}, {0}, {1}\n",
                            get_register_name(&res_reg),
                            get_register_name(register_id)
                        );
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
