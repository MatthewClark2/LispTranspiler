use crate::ast::Statement::*;
use crate::ast::Value::*;
use crate::lex::{Token, TokenValue::*};
use crate::parse::ParseTree;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use crate::ast::Scope::Global;

#[derive(Clone, Debug)]
pub enum ASTNode {
    Value(Value),
    Statement(Statement),
}

impl ASTNode {
    fn as_value(&self) -> &Value {
        match self {
            ASTNode::Value(v) => v,
            ASTNode::Statement(_) => panic!("Illegal conversion to value."),
        }
    }

    fn as_statement(&self) -> &Statement {
        match self {
            ASTNode::Statement(s) => s,
            ASTNode::Value(_) => panic!("Illegal conversion to value."),
        }
    }
}

pub fn construct_ast(parse_tree: &Vec<ParseTree>) -> Result<Vec<ASTNode>, (u32, String)> {
    let mut ast = Vec::new();

    for tree in parse_tree {
        ast.push(ASTNode::try_from(tree)?);
    }

    Ok(ast)
}

impl From<Token> for ASTNode {
    fn from(t: Token) -> Self {
        Self::Value(Literal(t.clone()))
    }
}

impl TryFrom<&ParseTree> for ASTNode {
    type Error = (u32, String);

    fn try_from(tree: &ParseTree) -> Result<Self, Self::Error> {
        match &tree {
            ParseTree::Leaf(t) => Ok(Self::from(t.clone())),
            ParseTree::Branch(elems, start, _stop) => {
                if elems.len() == 0 {
                    return Err((
                        *start,
                        String::from("Empty lists are unsupported as syntax elements."),
                    ));
                }

                return match &elems[0] {
                    ParseTree::Leaf(Token {
                                        line,
                                        value: Symbol(s),
                                    }) if &s[..] == "if" => {
                        if elems.len() != 4 {
                            return Err((
                                *line,
                                String::from(format!(
                                    "Expected exactly 3 arguments in `if` special form. Found {}.",
                                    elems.len()
                                )),
                            ));
                        }

                        let cond = Self::try_from(&elems[1])?;
                        let if_true = Self::try_from(&elems[2])?;
                        let if_false = Self::try_from(&elems[3])?;

                        match (cond, if_true, if_false) {
                            (ASTNode::Value(a), ASTNode::Value(b), ASTNode::Value(c)) => Ok(ASTNode::Value(Value::Condition(Box::new(a.clone()), Box::new(b.clone()), Box::new(c.clone())))),
                            _ => Err((*line, String::from("Expected values for condition, true, and false branches of condition.")))
                        }
                    }
                    ParseTree::Leaf(Token {
                                        line,
                                        value: Symbol(s),
                                    }) if &s[..] == "define" => {
                        if elems.len() != 3 {
                            return Err((*line, String::from(format!("Expected exactly 2 arguments in `define` special form. Found {}.", elems.len() - 1))));
                        }

                        let defined = Self::try_from(&elems[1])?;
                        let value = Self::try_from(&elems[2])?;

                        match (defined, value) {
                            (
                                ASTNode::Value(Literal(Token {
                                                           value: Symbol(s), ..
                                                       })),
                                ASTNode::Value(v),
                            ) => Ok(ASTNode::Statement(Definition(s.clone(), v.clone()))),
                            (
                                ASTNode::Value(Literal(Token {
                                                           value: Symbol(_s),
                                                           line,
                                                       })),
                                _,
                            ) => Err((line, String::from("Can only assign a symbol to a value."))),
                            (_, ASTNode::Value(_v)) => {
                                Err((*line, String::from("Can only assign a value to a symbol.")))
                            }
                            _ => Err((*line, String::from("Invalid definition."))),
                        }
                    }
                    ParseTree::Leaf(t) => match &t {
                        Token {
                            value: Symbol(_s),
                            line,
                        } => {
                            let mut args = Vec::new();

                            for subtree in &elems[1..] {
                                match Self::try_from(subtree)? {
                                    ASTNode::Value(v) => args.push(v),
                                    _ => {
                                        return Err((
                                            *line,
                                            String::from("All arguments in call should be values."),
                                        ));
                                    }
                                }
                            }
                            let args: Vec<Result<Self, Self::Error>> =
                                (&elems[1..]).iter().map(Self::try_from).collect();

                            let mut values = Vec::new();
                            for arg in args {
                                match arg {
                                    Ok(ASTNode::Value(v)) => (values.push(v.clone())),
                                    Ok(_) => return Err((*line, format!("Expected a value to be passed as an argument. Found: {:?}.", &elems[0]).to_string())),
                                    _ => return arg,
                                }
                            }
                            Ok(ASTNode::Value(Call(Box::new(Literal(t.clone())), values)))
                        }
                        _ => Err((
                            t.line(),
                            String::from("Symbols are the only literal value that may be invoked."),
                        )),
                    },
                    ParseTree::Branch(_elems, start, _stop) => {
                        // TODO(matthew-c21): Later, it should be possible to invoke lambda special
                        //  forms as well as functions that may return functions.
                        Err((
                            *start,
                            String::from("Compound forms cannot be used as function calls."),
                        ))
                    }
                };
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    // Should only hold valued tokens. Anything else should be removed during the parsing step.
    Literal(Token),

    // obviously callee and arguments
    Call(Box<Value>, Vec<Value>),

    // condition, value if true, value if false
    Condition(Box<Value>, Box<Value>, Box<Value>),
}

#[derive(Clone, Debug)]
pub enum Statement {
    // name, value, scope, is_redefinition
    // Definition(String, Value, Scope, bool),
    Definition(String, Value),
    Redefinition(String, Value),
    Declaration(String),
    ExpandedCondition(Value, Vec<ASTNode>, Vec<ASTNode>),
}

pub trait ASTVisitor<T> {
    fn visit(&self, ast: &ASTNode, sym_table: &mut SymbolTable) -> T {
        self.try_visit(ast, sym_table).unwrap()
    }

    fn try_visit(&self, ast: &ASTNode, sym_table: &mut SymbolTable) -> Result<T, (u32, String)>;
}

#[derive(Copy, Clone)]
struct Gensym {
    counter: u64,
}

impl Gensym {
    fn new() -> Self {
        Gensym { counter: 0 }
    }

    fn gen(&mut self, symbol: &str, prefix: Option<&str>) -> String {
        self.counter += 1;
        let s = Self::convert(symbol);
        let prefix = if prefix.is_none() {
            String::from("")
        } else {
            format!("_{}", prefix.unwrap()).as_str().to_owned()
        };

        format!("gensym{}{}_{}", self.counter, prefix, s).to_string()
    }

    /// Transform a non-C compliant symbol into a C compliant one.
    fn convert(name: &str) -> String {
        let mut output = String::new();

        if name.chars().nth(0).unwrap().is_digit(10) {
            output.push('_');
        }

        for c in name.chars() {
            let s = match c {
                '*' => "_times_",
                '$' => "_dollar_",
                '+' => "_plus_",
                '-' => "_minus_",
                '!' => "_excl_",
                '?' => "_question_",
                '/' => "_div_",
                '%' => "_mod_",
                '&' => "_amp_",
                '^' => "_caret_",
                '~' => "_tilde_",
                '<' => "_less_",
                '>' => "_great_",
                '=' => "_equal_",
                '@' => "_at_",
                _ => {
                    output.push(c);
                    ""
                }
            };

            output.push_str(s);
        }

        output
    }
}

pub struct FunctionUnfurl;

impl ASTVisitor<Vec<ASTNode>> for FunctionUnfurl {
    fn try_visit(
        &self,
        ast: &ASTNode,
        sym_table: &mut SymbolTable,
    ) -> Result<Vec<ASTNode>, (u32, String)> {
        let mut mapping: Vec<Value> = Vec::new();
        let mut result = Vec::new();

        match ast {
            ASTNode::Value(Call(_, args)) => {
                for arg in args {
                    match arg {
                        Call(_, _args) => {
                            let subexpansion =
                                self.try_visit(&ASTNode::Value(arg.clone()), sym_table)?;
                            assert!(subexpansion.len() > 0);

                            // All the preliminary statements are definitions.
                            for statement in &subexpansion[0..subexpansion.len() - 1] {
                                result.push(statement.clone())
                            }

                            // Create a new definition, then add it to the end of the list.
                            let s = sym_table.generate("function_unwrap", Scope::Global);

                            // This does lose information, but the code should be syntactically
                            //  correct at this stage, and the information isn't kept for runtime
                            //  debugging.
                            mapping.push(Value::Literal(Token::from(Symbol(s.clone()))));
                            let c = subexpansion.last().unwrap();
                            if let ASTNode::Value(v) = c {
                                result.push(ASTNode::Statement(Definition(s, v.clone())))
                            }
                        }
                        _ => mapping.push(arg.clone()), // It isn't a function call, so we don't deal with it here.
                    }
                }
            }
            _ => return Ok(vec![ast.clone()]),
        }

        // Finally, create a new function call based on the unrolled variant. The condition is
        //  redundant, but it's cleaner than trying to get the information earlier.
        if let ASTNode::Value(Call(callee, _)) = ast {
            result.push(ASTNode::Value(Call(callee.clone(), mapping)))
        }

        Ok(result)
    }
}

pub struct ConditionUnroll;

impl ConditionUnroll {
    fn split_condition(
        &self,
        c: &Box<Value>,
        sym_table: &mut SymbolTable,
    ) -> (Value, Vec<ASTNode>) {
        let mut prefix = self.visit(&ASTNode::Value(*c.clone()), sym_table);
        let value = prefix.pop().unwrap();

        if let ASTNode::Value(v) = value {
            (v, prefix)
        } else {
            panic!("Illegal program state.")
        }
    }
}

impl ASTVisitor<Vec<ASTNode>> for ConditionUnroll {
    fn try_visit(
        &self,
        ast: &ASTNode,
        sym_table: &mut SymbolTable,
    ) -> Result<Vec<ASTNode>, (u32, String)> {
        let mut output: Vec<ASTNode> = Vec::new();
        let mut iftrue: Vec<ASTNode>;
        let mut iffalse: Vec<ASTNode>;

        match ast {
            // Check for nested conditions in top level ones.
            ASTNode::Value(Condition(c, t, f)) => {
                let output_name = sym_table.generate("conditional_value", Scope::Global);
                output.push(ASTNode::Statement(Declaration(output_name.clone())));

                iftrue = self.try_visit(&ASTNode::Value(*t.clone()), sym_table)?;
                iffalse = self.try_visit(&ASTNode::Value(*f.clone()), sym_table)?;
                let condition;

                match **c {
                    Condition(_, _, _) => {
                        let (value, mut prefix) = self.split_condition(c, sym_table);
                        condition = Box::new(value);
                        output.append(&mut prefix);
                    }
                    _ => condition = c.clone(),
                }

                // Update each branch to assign to the output variable.
                let true_value = iftrue.pop().unwrap();
                let false_value = iffalse.pop().unwrap();

                iftrue.push(ASTNode::Statement(Definition(output_name.clone(), true_value.as_value().to_owned())));
                iffalse.push(ASTNode::Statement(Definition(output_name.clone(), false_value.as_value().to_owned())));

                output.push(ASTNode::Statement(ExpandedCondition(
                    *condition, iftrue, iffalse,
                )));
                output.push(ASTNode::Value(Literal(Token::from(Symbol(
                    output_name.clone(),
                )))));

                Ok(output)
            }
            // Handle the case of a condition used as a value for a definition.
            ASTNode::Statement(Definition(name, Condition(c, t, f))) => {
                let mut prefix = self.try_visit(
                    &ASTNode::Value(Condition(c.clone(), t.clone(), f.clone())),
                    sym_table,
                )?;
                let value = prefix.pop().unwrap();

                output.push(ASTNode::Statement(Definition(
                    name.clone(),
                    value.as_value().clone(),
                )));

                Ok(output)
            }
            // Handle the case of a condition inside a function call.
            ASTNode::Value(Call(callee, args)) => {
                let mut new_args = Vec::new();

                for arg in args {
                    let mut expansion = self.try_visit(&ASTNode::Value(arg.clone()), sym_table)?;

                    // No matter what, the final element is necessarily exists and is a Value.
                    new_args.push(expansion.pop().unwrap());

                    // The value of the condition has been emplaced, so we take the rest of the
                    // expansion and put it before the newly formed function call.
                    output.append(&mut expansion);
                }

                let new_args: Vec<Value> = new_args.iter().map(|node| node.as_value().clone()).collect();

                output.push(ASTNode::Value(Call(callee.clone(), new_args)));

                return Ok(output);
            }
            _ => Ok(vec![ast.clone()]),
        }
    }
}

pub struct SymbolValidation;

impl ASTVisitor<ASTNode> for SymbolValidation {
    fn try_visit(&self, ast: &ASTNode, sym_table: &mut SymbolTable) -> Result<ASTNode, (u32, String)> {
        match ast {
            ASTNode::Statement(Statement::ExpandedCondition(v, t, f)) => {
                let v = self.try_visit(&ASTNode::Value(v.clone()), sym_table)?;
                let mut mt = Vec::new();
                let mut mf = Vec::new();

                for x in t {
                    mt.push(self.try_visit(x, sym_table)?);
                };

                for x in f {
                    mf.push(self.try_visit(x, sym_table)?);
                };

                Ok(ASTNode::Statement(ExpandedCondition(v.as_value().clone(), mt, mf)))
            }
            ASTNode::Statement(Definition(name, value)) => {
                let value = self.try_visit(&ASTNode::Value(value.clone()), sym_table)?;

                if !sym_table.contains(name.as_str()) {
                    sym_table.register(name.as_str(), Global);
                    Ok(ASTNode::Statement(Definition(name.clone(), value.as_value().to_owned())))
                } else {
                    Ok(ASTNode::Statement(Redefinition(name.clone(), value.as_value().to_owned())))
                }
            }
            ASTNode::Statement(Redefinition(name, value)) => {
                if !sym_table.contains(name.as_str()) {
                    Err((0, format!("Cannot redefine symbol `{}` as it does not exist. Contact the developer.", name)))
                } else {
                    let value = self.try_visit(&ASTNode::Value(value.clone()), sym_table)?;
                    Ok(ASTNode::Statement(Redefinition(name.clone(), value.as_value().to_owned())))
                }
            }
            ASTNode::Statement(Declaration(name)) => {
                if sym_table.contains(name.as_str()) {
                    Err((0, format!("Redeclaration of existing name `{}`. Contact the developer.", name)))
                } else {
                    Ok(ast.clone())
                }
            }
            ASTNode::Value(Condition(c, t, f)) => {
                let c = self.try_visit(&ASTNode::Value(c.as_ref().clone()), sym_table)?;
                let t = self.try_visit(&ASTNode::Value(t.as_ref().clone()), sym_table)?;
                let f = self.try_visit(&ASTNode::Value(f.as_ref().clone()), sym_table)?;

                Ok(ASTNode::Value(Condition(Box::new(c.as_value().to_owned()), Box::new(t.as_value().to_owned()), Box::new(f.as_value().to_owned()))))
            }
            ASTNode::Value(Call(callee, args)) => {
                let callee = self.try_visit(&ASTNode::Value(callee.as_ref().clone()), sym_table)?;
                let mut margs = Vec::new();

                for arg in args {
                    margs.push((self.try_visit(&ASTNode::Value(arg.clone()), sym_table)?).as_value().to_owned());
                }

                Ok(ASTNode::Value(Call(Box::new(callee.as_value().to_owned()), margs)))
            }
            ASTNode::Value(Literal(t)) => {
                if let Symbol(name) = t.value() {
                    if !sym_table.contains(name.as_str()) {
                        return Err((t.line(), format!("Use of undefined variable: {}.", name)))
                    }
                }

                Ok(ast.clone())
            }
        }
    }
}

#[derive(Clone)]
pub struct SymbolTable {
    natives: HashMap<String, String>,
    defs: HashMap<String, SymbolTableEntry>,
    gensym: Gensym,
}

impl SymbolTable {
    /// Strictly used for creating new variable names. Does not actually assign them within the
    /// table.
    fn generate(&mut self, base_name: &str, _scope: Scope) -> String {
        self.gensym.gen(base_name, None)
    }

    // Adds a new name to the table, generating a SymbolTableEntry containing the corresponding C
    // variable name.
    fn register(&mut self, name: &str, scope: Scope) {
        let key = name.to_string();
        let c_name = Gensym::convert(&key);

        if !self.defs.contains_key(&key) {
            self.defs.insert(key, SymbolTableEntry::from(c_name, scope));
        }
    }

    fn contains(&self, name: &str) -> bool {
        self.contains_fn(name) || self.contains_obj(name)
    }

    fn contains_fn(&self, name: &str) -> bool {
        self.natives.contains_key(name)
    }

    fn contains_obj(&self, name: &str) -> bool {
        self.defs.contains_key(name)
    }

    fn get(&self, name: &str) -> Option<&String> {
        if self.natives.contains_key(name) {
            self.natives.get(name)
        } else {
            self.defs.get(name).map(|n| n.c_name())
        }
    }

    pub fn dummy() -> Self {
        Self {
            natives: HashMap::new(),
            defs: HashMap::new(),
            gensym: Gensym::new(),
        }
    }
}

#[derive(Clone)]
struct SymbolTableEntry {
    c_name: String,
    scope: Scope,
}

impl SymbolTableEntry {
    fn from(c_name: String, scope: Scope) -> Self {
        Self { c_name, scope }
    }

    fn c_name(&self) -> &String {
        &self.c_name
    }

    fn scope(&self) -> &Scope {
        &self.scope
    }
}

#[derive(Clone)]
pub enum Scope {
    /// File global scoping. Available in all subsequent code, as well as in previously defined functions.
    Global,

    /// Local variable, such as that in a `let` expression. Gets an associated tag to match it with.
    Local(String),

    /// Function parameters.
    Function(String),
}

#[cfg(test)]
mod test_utils {
    use crate::ast::*;
    use crate::lex::start;
    use crate::parse::parse;

    pub fn force_from(input: &str) -> Vec<ASTNode> {
        parse(&start(input).unwrap())
            .unwrap()
            .iter()
            .map(ASTNode::try_from)
            .map(Result::unwrap)
            .collect()
    }

    pub fn from_line(input: &str) -> Result<ASTNode, (u32, String)> {
        let mut ast: Vec<Result<ASTNode, (u32, String)>> = parse(&start(input).unwrap())
            .unwrap()
            .iter()
            .map(ASTNode::try_from)
            .collect();

        assert_eq!(1, ast.len());

        ast.remove(0)
    }
}

#[cfg(test)]
mod visitor_tests {
    use crate::ast::test_utils::force_from;
    use crate::ast::*;
    use crate::ast::*;
    use crate::lex::TokenValue::*;
    use crate::lex::{start, TokenValue};
    use crate::parse::parse;

    #[test]
    fn basic_call_unroll() {
        let ast = force_from("(format \"hello\" (+ 1 1))");
        let ast = FunctionUnfurl
            .try_visit(&ast[0], &mut SymbolTable::dummy())
            .unwrap();

        assert_eq!(ast.len(), 2);

        if let ASTNode::Statement(Definition(_name, value)) = &ast[0] {
            if let Call(_plus, args) = value {
                for arg in args {
                    if let Literal(t) = arg {
                        assert_eq!(t.value(), Int(1))
                    } else {
                        panic!()
                    }
                }
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }

    #[test]
    fn simple_invalid_symbol() {
        let ast = force_from("hello");
        assert_eq!(1, ast.len());

        let sv = SymbolValidation;
        let mut st = SymbolTable::dummy();

        let node = sv.try_visit(&ast[0], &mut st);

        assert!(node.is_err());
    }

    #[test]
    fn invalid_symbol_in_condition() {
        let ast = force_from("(if something :hello :world)");
        assert_eq!(1, ast.len());

        let sv = SymbolValidation;
        let mut st = SymbolTable::dummy();

        let node = sv.try_visit(&ast[0], &mut st);

        assert!(node.is_err());
    }

    #[test]
    fn invalid_symbol_in_true_branch() {
        let ast = force_from("(if :something hello :world)");
        assert_eq!(1, ast.len());

        let sv = SymbolValidation;
        let mut st = SymbolTable::dummy();

        let node = sv.try_visit(&ast[0], &mut st);

        assert!(node.is_err());
    }

    #[test]
    fn invalid_symbol_in_false_branch() {
        let ast = force_from("(if :something :hello world)");
        assert_eq!(1, ast.len());

        let sv = SymbolValidation;
        let mut st = SymbolTable::dummy();

        let node = sv.try_visit(&ast[0], &mut st);

        assert!(node.is_err());
    }

    fn invalid_symbol_in_definition() {
        let ast = force_from("(define hello world)");
        assert_eq!(1, ast.len());

        let sv = SymbolValidation;
        let mut st = SymbolTable::dummy();

        let node = sv.try_visit(&ast[0], &mut st);

        assert!(node.is_err());
    }

    fn symbol_valid_after_definition() {
        let ast = force_from("(define hello :world) hello");
        assert_eq!(2, ast.len());

        let sv = SymbolValidation;
        let mut st = SymbolTable::dummy();

        let node = sv.try_visit(&ast[0], &mut st);

        assert!(node.is_ok());
    }

    fn symbol_valid_after_redefinition() {
        let ast = force_from(
            "(define hello :world) hello\
                   (define hello :goodbye) hello"
        );
        assert_eq!(2, ast.len());

        let sv = SymbolValidation;
        let mut st = SymbolTable::dummy();

        let node = sv.try_visit(&ast[0], &mut st);

        assert!(node.is_ok());
    }
}

#[cfg(test)]
mod ast_tests {
    use crate::ast::test_utils::*;
    use crate::ast::*;
    use crate::lex::TokenValue::*;
    use crate::lex::{start, TokenValue};
    use crate::parse::parse;

    #[test]
    fn from_literal() {
        let ast = force_from("hello");
        assert_eq!(1, ast.len());

        if let ASTNode::Value(Literal(t)) = &ast[0] {
            assert_eq!(Symbol(String::from("hello")), t.value())
        } else {
            panic!("Failed AST generation")
        }
    }

    #[test]
    fn from_define() {
        let ast = force_from("(define foobar \"foo bar\")");
        assert_eq!(1, ast.len());

        if let ASTNode::Statement(Definition(name, value)) = &ast[0] {
            assert_eq!("foobar", name.as_str());

            match value {
                Literal(t) => assert_eq!(TokenValue::Str("foo bar".to_string()), t.value()),
                _ => panic!(),
            }
        } else {
            panic!()
        }
    }

    #[test]
    fn define_non_symbol() {
        let result: Result<ASTNode, (u32, String)> = from_line("(define 123 456)");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!("Can only assign a value to a symbol.", msg.as_str())
        }
    }

    #[test]
    fn define_non_value() {
        let result: Result<ASTNode, (u32, String)> = from_line("(define a (define b 1))");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!("Can only assign a symbol to a value.", msg.as_str())
        }
    }

    #[test]
    fn fully_malformed_define() {
        let result: Result<ASTNode, (u32, String)> = from_line("(define 8 (define c 1))");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!("Invalid definition.", msg.as_str())
        }
    }

    #[test]
    fn wrong_number_define() {
        let result: Result<ASTNode, (u32, String)> = from_line("(define a 1 2)");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected exactly 2 arguments in `define` special form. Found 3.",
                msg.as_str()
            )
        }

        let result: Result<ASTNode, (u32, String)> = from_line("(define a)");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected exactly 2 arguments in `define` special form. Found 1.",
                msg.as_str()
            )
        }

        let result: Result<ASTNode, (u32, String)> = from_line("(define)");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected exactly 2 arguments in `define` special form. Found 0.",
                msg.as_str()
            )
        }
    }

    #[test]
    fn wrong_number_condition() {}

    #[test]
    fn conditional_non_values() {}

    #[test]
    fn from_condition() {
        let ast = force_from("(if #t \"true\" \"false\")");
        assert_eq!(1, ast.len());

        if let ASTNode::Value(Condition(a, b, c)) = &ast[0] {
            if let (Literal(cond), Literal(if_true), Literal(if_false)) =
            (a.as_ref(), b.as_ref(), c.as_ref())
            {
                assert_eq!(True, cond.value());
                assert_eq!(Str("true".to_string()), if_true.value());
                assert_eq!(Str("false".to_string()), if_false.value());
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }

    #[test]
    fn from_malformed_condition() {}

    #[test]
    // TODO(matthew-c21): After adding all the relevant listeners, replace this with something else.
    fn basic_comprehensive() {}

    /*
    Still need:

    symbol table tracing and relevant errors
    validation of called functions
    function call unfurling
    conditional unfurling
     */
}

/* AST Construction

1. Split off special forms via series of visitors. Each should ensure that the special form is
   correctly formed.
   - The SymbolTable should start being built when splitting off Definitions.
2. Unfurl nested function calls.
3. Trace through the program to ensure that symbols are properly utilized.
4. Change any re-defined variables to ensure that they refer to the correct value in the scope in
   which they have been redefined.

 */
