use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Variable {
    // 对于int类型的变量，存储其在生成的koopaIR中临时变量的名字（e.g. @x）
    INT(String),
    // 对于int类型的常量，需要保存的信息只有它的值
    ConstINT(i32),
}

/// CompUnit ::= FuncDef
#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}

/// FuncDef ::= ItemType IDENT "(" ")" Block
#[derive(Debug, Clone)]
pub struct FuncDef {
    pub func_type: ItemType,
    pub id: String,
    pub block: Block,
}

/// ItemType ::= "int"
#[derive(Debug, PartialEq, Clone)]
pub enum ItemType {
    Int,
    Double,
}

/// Block ::= "{" {BlockItem} "}"
#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub items: Vec<BlockItem>,
    pub vars_table: HashMap<String, i32>,
}

/// BlockItem ::= Decl | Stmt
#[derive(Debug, PartialEq, Clone)]
pub enum BlockItem {
    Decl(Decl),
    Stmt(Stmt),
}

/// Decl ::= ConstDecl | VarDecl
#[derive(Debug, PartialEq, Clone)]
pub enum Decl {
    ConstDecl(ConstDecl),
    VarDecl(VarDecl),
}

/// VarDecl ::= BType VarDef {"," VarDef} ";"
#[derive(Debug, PartialEq, Clone)]
pub struct VarDecl {
    pub b_type: ItemType,
    pub var_defs: Vec<VarDef>,
}

/// ConstDecl ::= "const" ItemType ConstDef {"," ConstDef} ";"
#[derive(Debug, PartialEq, Clone)]
pub struct ConstDecl {
    pub b_type: ItemType,
    pub const_defs: Vec<ConstDef>,
}

/// ConstDef ::= IDENT "=" ConstInitVal
#[derive(Debug, PartialEq, Clone)]
pub struct ConstDef {
    pub ident: String,
    pub const_init_val: ConstInitVal,
}

/// VarDef ::= IDENT | IDENT "=" InitVal
#[derive(Debug, PartialEq, Clone)]
pub enum VarDef {
    Decl(String),
    Def((String, InitVal)),
}

/// InitVal ::= Exp
#[derive(Debug, PartialEq, Clone)]
pub struct InitVal {
    pub exp: Exp,
}

/// ConstInitVal ::= ConstExp
#[derive(Debug, PartialEq, Clone)]
pub struct ConstInitVal {
    pub const_exp: ConstExp,
}

/// Stmt ::= LVal "=" Exp ";"
///       | "return" Exp ";"
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Return(Exp),
    Assign((String, Exp)),
}

/// Exp ::= LOrExp
#[derive(Debug, PartialEq, Clone)]
pub struct Exp {
    pub lor_exp: Box<LOrExp>,
}

/// ConstExp ::= Exp
#[derive(Debug, PartialEq, Clone)]
pub struct ConstExp {
    pub exp: Box<Exp>,
}
/// PrimaryExp ::= "(" Exp ")" | Number | LVal
#[derive(Debug, PartialEq, Clone)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    Number(i32),
    LVal(String),
}

/// UnaryExp ::= PrimaryExp | UnaryOp UnaryExp
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryExp {
    PrimaryExp(Box<PrimaryExp>),
    UnaryExp((UnaryOp, Box<UnaryExp>)),
}

/// AddExp ::= MulExp | AddExp AddOp MulExp
#[derive(Debug, PartialEq, Clone)]
pub enum AddExp {
    MulExp(Box<MulExp>),
    AddExp((Box<AddExp>, AddOp, Box<MulExp>)),
}

/// MulExp ::= UnaryExp | MulExp MulOp UnaryExp
#[derive(Debug, PartialEq, Clone)]
pub enum MulExp {
    UnaryExp(Box<UnaryExp>),
    MulExp((Box<MulExp>, MulOp, Box<UnaryExp>)),
}

/// LOrExp ::= LAndExp | LOrExp "||" LAndExp
#[derive(Debug, PartialEq, Clone)]
pub enum LOrExp {
    LAndExp(Box<LAndExp>),
    LOrExp((Box<LOrExp>, Box<LAndExp>)),
}

/// LAndExp ::= EqExp | LAndExp "&&" EqExp
#[derive(Debug, PartialEq, Clone)]
pub enum LAndExp {
    EqExp(Box<EqExp>),
    LAndExp((Box<LAndExp>, Box<EqExp>)),
}

/// EqExp ::= RelExp | EqExp CmpOp RelExp
#[derive(Debug, PartialEq, Clone)]
pub enum EqExp {
    RelExp(Box<RelExp>),
    EqExp((Box<EqExp>, CmpOp, Box<RelExp>)),
}

/// RelExp ::= AddExp | RelExp CmpOp AddExp
#[derive(Debug, PartialEq, Clone)]
pub enum RelExp {
    AddExp(Box<AddExp>),
    CompExp((Box<RelExp>, CmpOp, Box<AddExp>)),
}

/// UnaryOp ::= "+" | "-" | "!"
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Passive,
    Negative,
    Inversion,
}

/// AddOp ::= "+" | "-"
#[derive(Debug, PartialEq, Clone)]
pub enum AddOp {
    Add,
    Sub,
}

/// MulOp ::= "*" | "/" | "%"
#[derive(Debug, PartialEq, Clone)]
pub enum MulOp {
    Multiple,
    Divide,
    Mod,
}

/// CmpOp ::= "==" | "!=" | "<" | ">" |"<=" | ">="
#[derive(Debug, PartialEq, Clone)]
pub enum CmpOp {
    Eq,
    NEq,
    Less,
    Grate,
    LessEq,
    GrateEq,
}
