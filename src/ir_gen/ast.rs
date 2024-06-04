use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Variable {
    // 对于int类型的变量，存储其在生成的koopaIR中临时变量的名字（e.g. @x）
    INT(String),
    // 对于int类型的常量，需要保存的信息只有它的值
    ConstINT(i32),
    // 对于函数对象，保存其koopaIR中的函数名（e.g. @main)及其返回值类型
    Func((String, ItemType, Vec<bool>)),
    // 对于数组（指针）对象，需要保存其名
    Array(String),
    Ptr((String, ItemType)),
}

/// CompUnit ::= [CompUnit] GlobalItem
#[derive(Debug, PartialEq, Clone)]
pub struct CompUnit {
    pub comp_unit: Box<Option<CompUnit>>,
    pub global_item: GlobalItem,
}

#[derive(Debug, PartialEq, Clone)]
pub enum GlobalItem {
    Func(FuncDef),
    Decl(Decl),
}

/// FuncDef ::= FuncType IDENT "(" [FuncFParams] ")" Block
#[derive(Debug, PartialEq, Clone)]
pub struct FuncDef {
    pub func_type: ItemType,
    pub id: String,
    pub func_f_params: Option<FuncFParams>,
    pub block: Block,
}

/// FuncFParams ::= FuncFParam {"," FuncFParam}
#[derive(Debug, PartialEq, Clone)]
pub struct FuncFParams {
    pub func_f_params: Vec<FuncFParam>,
}

/// FuncFParam  ::= BType IDENT
#[derive(Debug, PartialEq, Clone)]
pub struct FuncFParam {
    pub b_type: ItemType,
    pub id: String,
    pub dims: Option<Vec<ConstExp>>,
}

/// FuncRParams ::= Exp {"," Exp}
#[derive(Debug, PartialEq, Clone)]
pub struct FuncRParams {
    pub func_r_params: Vec<Exp>,
}

/// ItemType ::= "int"
#[derive(Debug, PartialEq, Clone)]
pub enum ItemType {
    Int,
    Void,
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
    pub dims: Vec<ConstExp>,
    pub const_init_val: ConstInitVal,
}

/// VarDef ::= IDENT | IDENT "=" InitVal
#[derive(Debug, PartialEq, Clone)]
pub enum VarDef {
    Decl((String, Vec<ConstExp>)),
    Def((String, Vec<ConstExp>, InitVal)),
}

/// InitVal ::= Exp
#[derive(Debug, PartialEq, Clone)]
pub enum InitVal {
    Exp(Exp),
    Array(Vec<InitVal>),
}

/// ConstInitVal ::= ConstExp
#[derive(Debug, PartialEq, Clone)]
pub enum ConstInitVal {
    Exp(ConstExp),
    Array(Vec<ConstInitVal>),
}

/// "if" "(" cond ")" then_stmt ["else" self_stmt]
#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub cond: Exp,
    pub then_stmt: Stmt,
    pub else_stmt: Option<Stmt>,
}

/// "if" "(" cond ")" then_stmt ["else" self_stmt]
#[derive(Debug, PartialEq, Clone)]
pub struct While {
    pub cond: Exp,
    pub body_stmt: Stmt,
}

/// Stmt ::= LVal "=" Exp ";"
///       | "return" Exp? ";"
///       | Exp? ";"
///       | "while" "(" Exp ")" Stmt
///       | "if" "(" Exp ")" Stmt ["else" Stmt]
///       | Block
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Return(Option<Exp>),
    Assign((LVal, Exp)),
    Exp(Option<Exp>),
    Block(Block),
    IF(Box<If>),
    WHILE(Box<While>),
    Break,
    Continue,
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
    LVal(LVal),
}

/// UnaryExp ::= PrimaryExp
///             | UnaryOp UnaryExp
///             | IDENT "(" [FuncRParams] ")"
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryExp {
    PrimaryExp(Box<PrimaryExp>),
    UnaryExp((UnaryOp, Box<UnaryExp>)),
    FuncItem((String, Option<FuncRParams>)),
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

/// LVal ::= IDENT {"[" Exp "]"}
#[derive(Debug, PartialEq, Clone)]
pub struct LVal {
    pub ident: String,
    pub indices: Vec<Exp>,
}
