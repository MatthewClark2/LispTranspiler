use crate::ast::{Statement::*, Value::*};
use crate::lex::{Token, TokenValue::*};
use crate::parse::ParseTree;
use json;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;

#[derive(Clone, Debug)]
pub enum ASTNode {
    Value(Value),
    Statement(Statement),
}

impl ASTNode {
    fn as_value(&self) -> &Value {
        match self {
            ASTNode::Value(v) => v,
            _ => panic!("Illegal conversion to value."),
        }
    }

    fn try_from_parse_tree(tree: &ParseTree, scope_id: &mut usize) -> Result<ASTNode, (u32, String)>  {
        match &tree {
            ParseTree::Leaf(t) => Ok(Self::from(t.clone())),
            ParseTree::Branch(elems, start, _stop, None) => {
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
                                    elems.len() - 1
                                )),
                            ));
                        }

                        let cond = Self::try_from_parse_tree(&elems[1], scope_id)?;
                        let if_true = Self::try_from_parse_tree(&elems[2], scope_id)?;
                        let if_false = Self::try_from_parse_tree(&elems[3], scope_id)?;

                        match (cond, if_true, if_false) {
                            (ASTNode::Value(a), ASTNode::Value(b), ASTNode::Value(c)) => Ok(ASTNode::Value(Value::Condition(Box::new(a.clone()), Box::new(b.clone()), Box::new(c.clone())))),
                            _ => Err((*line, String::from("Expected values for condition, true, and false branches of conditional expression.")))
                        }
                    }
                    ParseTree::Leaf(Token {
                                        line,
                                        value: Symbol(s),
                                    }) if &s[..] == "define" => {
                        if elems.len() != 3 {
                            return Err((*line, String::from(format!("Expected exactly 2 arguments in `define` special form. Found {}.", elems.len() - 1))));
                        }

                        let defined = Self::try_from_parse_tree(&elems[1], scope_id)?;
                        let value = Self::try_from_parse_tree(&elems[2], scope_id)?;

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
                    ParseTree::Leaf(Token {
                                        line,
                                        value: Symbol(s),
                                    }) if &s[..] == "lambda" => {
                        if elems.len() != 3 {
                            return Err((*line, format!("Expected exactly 2 arguments in `lambda` special form. Found {}.", elems.len() - 1)));
                        }

                        let mut names = Vec::new();
                        let mut vararg = None;

                        match &elems[1] {
                            ParseTree::Branch(args, start, _stop, varg) => {
                                for arg in args {
                                    if let ParseTree::Leaf(t) = arg {
                                        match t.value() {
                                            Symbol(n) => names.push(n.clone()),
                                            _ => return Err((*start, "All elements in first argument to `lambda` special form should be symbols.".to_string()))
                                        }
                                    } else {
                                        return Err((*start, "All elements in first argument to `lambda` special form should be symbols.".to_string()));
                                    }
                                }

                                if let Some(b) = varg {
                                    match b.as_ref() {
                                        ParseTree::Leaf(t) => {
                                            if let Symbol(n) = t.value() {
                                                vararg = Some(n.clone());
                                            } else {
                                                return Err((t.line(), "Expected a symbol to be used as a vararg.".to_string()));
                                            }
                                        }
                                        ParseTree::Branch(_, start, _, _) => {
                                            return Err((*start, "All elements in first argument to `lambda` special form should be symbols.".to_string()))
                                        }
                                    }
                                }
                            }
                            _ => {
                                return Err((
                                    *start,
                                    "Expected arglist in second position of `lambda` special form."
                                        .to_string(),
                                ));
                            }
                        }

                        let body = Self::try_from_parse_tree(&elems[2], scope_id)?;

                        if let ASTNode::Statement(_) = body {
                            return Err((
                                *line,
                                "Expected final argument to `lambda` special form to be a value."
                                    .to_string(),
                            ));
                        }

                        *scope_id += 1;
                        Ok(ASTNode::Value(Lambda(names, vararg, vec![body], *scope_id)))
                    }
                    ParseTree::Leaf(t) => match &t {
                        Token {
                            value: Symbol(s),
                            line,
                        } => {
                            let mut args = Vec::new();

                            for subtree in &elems[1..] {
                                match Self::try_from_parse_tree(subtree, scope_id)? {
                                    ASTNode::Value(v) => args.push(v),
                                    _ => {
                                        return Err((
                                            *line,
                                            String::from("All arguments in call should be values."),
                                        ));
                                    }
                                }
                            }
                            let args: Vec<Result<Self, (u32, String)>> =
                                (&elems[1..]).iter().map(|e| Self::try_from_parse_tree(e, scope_id)).collect();

                            let mut values = Vec::new();
                            for arg in args {
                                match arg {
                                    Ok(ASTNode::Value(v)) => (values.push(v.clone())),
                                    Ok(_) => {
                                        return Err((*line, format!("Expected a value to be passed as an argument. Found: {:?}.", &elems[0]).to_string()))
                                    },
                                    _ => return arg,
                                }
                            }
                            Ok(ASTNode::Value(Call(s.clone(), values)))
                        }
                        _ => Err((
                            t.line(),
                            String::from("Symbols are the only literal value that may be invoked."),
                        )),
                    },
                    ParseTree::Branch(_elems, start, _stop, _) => Err((
                        *start,
                        String::from("Compound forms cannot be used as function calls."),
                    )),
                };
            }
            ParseTree::Branch(_, start, _, _) => {
                Err((*start, String::from("Unexpected syntax token `.`.")))
            }
        }
    }
}

pub fn construct_ast(parse_tree: &Vec<ParseTree>) -> Result<Vec<ASTNode>, (u32, String)> {
    let mut ast = Vec::new();
    let mut scope_count: usize = 0;

    for tree in parse_tree {
        ast.push(ASTNode::try_from((tree, &mut scope_count))?);
    }

    Ok(ast)
}

impl From<Token> for ASTNode {
    fn from(t: Token) -> Self {
        Self::Value(Literal(t.clone()))
    }
}

impl TryFrom<(&ParseTree, &mut usize)> for ASTNode {
    type Error = (u32, String);

    fn try_from(data: (&ParseTree, &mut usize)) -> Result<Self, Self::Error> {
        let (tree, scope_id) = data;
        Self::try_from_parse_tree(tree, scope_id)
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    // Should only hold valued tokens. Anything else should be removed during the parsing step.
    Literal(Token),

    // obviously callee and arguments
    Call(String, Vec<Value>),

    // required_args, vararg, body, scope ID
    // The body should be a single element on creation, but may be expanded as a result of other
    // visitors.
    Lambda(Vec<String>, Option<String>, Vec<ASTNode>, usize),

    // condition, value if true, value if false
    Condition(Box<Value>, Box<Value>, Box<Value>),
}

#[derive(Clone, Debug)]
pub enum Statement {
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
            ASTNode::Value(Call(callee, args)) => {
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
                            let s = sym_table.generate("function_unwrap");

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

                result.push(ASTNode::Value(Call(callee.clone(), mapping)))
            }
            ASTNode::Value(Lambda(args, vararg, body, scope_id)) => {
                let mut new_body = Vec::new();

                for line in body {
                    new_body.append(&mut self.try_visit(line, sym_table)?)
                }

                result.push(ASTNode::Value(Lambda(
                    args.clone(),
                    vararg.clone(),
                    new_body,
                    *scope_id,
                )));
            }
            ASTNode::Value(Condition(c, t, f)) => {
                let mut cond = self.try_visit(&ASTNode::Value(c.as_ref().clone()), sym_table)?;
                let c = cond.pop().unwrap();
                result.append(&mut cond);
                result.push(ASTNode::Value(Condition(
                    Box::from(c.as_value().to_owned()),
                    t.clone(),
                    f.clone(),
                )));
            }
            _ => return Ok(vec![ast.clone()]),
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
                let output_name = sym_table.generate("conditional_value");
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

                iftrue.push(ASTNode::Statement(Redefinition(
                    output_name.clone(),
                    true_value.as_value().to_owned(),
                )));
                iffalse.push(ASTNode::Statement(Redefinition(
                    output_name.clone(),
                    false_value.as_value().to_owned(),
                )));

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

                let new_args: Vec<Value> = new_args
                    .iter()
                    .map(|node| node.as_value().clone())
                    .collect();

                output.push(ASTNode::Value(Call(callee.clone(), new_args)));

                return Ok(output);
            }
            // Condition in lambda body.
            ASTNode::Value(Lambda(args, vararg, body, scope_id)) => {
                let mut new_body = Vec::new();

                for line in body {
                    new_body.append(&mut self.try_visit(line, sym_table)?);
                }

                Ok(vec![ASTNode::Value(Lambda(
                    args.clone(),
                    vararg.clone(),
                    new_body,
                    *scope_id,
                ))])
            }
            _ => Ok(vec![ast.clone()]),
        }
    }
}

pub struct SymbolValidation;

impl SymbolValidation {
    fn try_visit_aux(
        &self,
        ast: &ASTNode,
        sym_table: &mut SymbolTable,
        scope_ids: &mut Vec<usize>,
    ) -> Result<ASTNode, (u32, String)> {
        match ast {
            ASTNode::Statement(Statement::ExpandedCondition(v, t, f)) => {
                let v = self.try_visit_aux(&ASTNode::Value(v.clone()), sym_table, scope_ids)?;
                let mut mt = Vec::new();
                let mut mf = Vec::new();

                for x in t {
                    mt.push(self.try_visit_aux(x, sym_table, scope_ids)?);
                }

                for x in f {
                    mf.push(self.try_visit_aux(x, sym_table, scope_ids)?);
                }

                Ok(ASTNode::Statement(ExpandedCondition(
                    v.as_value().clone(),
                    mt,
                    mf,
                )))
            }
            ASTNode::Statement(Definition(name, value)) => {
                let value =
                    self.try_visit_aux(&ASTNode::Value(value.clone()), sym_table, scope_ids)?;

                // No scope IDs are required because definitions are only allowed at the top level.
                if !sym_table.get(name.as_str(), None).is_some() {
                    sym_table.register(name.as_str(), None);
                    Ok(ASTNode::Statement(Definition(
                        name.clone(),
                        value.as_value().to_owned(),
                    )))
                } else {
                    Ok(ASTNode::Statement(Redefinition(
                        name.clone(),
                        value.as_value().to_owned(),
                    )))
                }
            }
            ASTNode::Statement(Redefinition(name, value)) => {
                // No scope IDs are required because definitions are only allowed at the top level.
                if !sym_table.get(name.as_str(), None).is_some() {
                    Err((0, format!("Cannot redefine symbol `{}` as it does not exist. Contact the developer.", name)))
                } else {
                    let value =
                        self.try_visit_aux(&ASTNode::Value(value.clone()), sym_table, scope_ids)?;
                    Ok(ASTNode::Statement(Redefinition(
                        name.clone(),
                        value.as_value().to_owned(),
                    )))
                }
            }
            ASTNode::Statement(Declaration(name)) => {
                // No scope IDs are required because definitions are only allowed at the top level.
                if sym_table.get(name.as_str(), None).is_some() {
                    Err((
                        0,
                        format!(
                            "Redeclaration of existing name `{}`. Contact the developer.",
                            name
                        ),
                    ))
                } else {
                    // Register it.
                    sym_table.register(name.as_str(), None);
                    Ok(ast.clone())
                }
            }
            ASTNode::Value(Condition(c, t, f)) => {
                let c =
                    self.try_visit_aux(&ASTNode::Value(c.as_ref().clone()), sym_table, scope_ids)?;
                let t =
                    self.try_visit_aux(&ASTNode::Value(t.as_ref().clone()), sym_table, scope_ids)?;
                let f =
                    self.try_visit_aux(&ASTNode::Value(f.as_ref().clone()), sym_table, scope_ids)?;

                Ok(ASTNode::Value(Condition(
                    Box::new(c.as_value().to_owned()),
                    Box::new(t.as_value().to_owned()),
                    Box::new(f.as_value().to_owned()),
                )))
            }
            ASTNode::Value(Call(callee, args)) => {
                let mut margs = Vec::new();

                for arg in args {
                    margs.push(
                        (self.try_visit_aux(&ASTNode::Value(arg.clone()), sym_table, scope_ids)?)
                            .as_value()
                            .to_owned(),
                    );
                }

                Ok(ASTNode::Value(Call(callee.clone(), margs)))
            }
            ASTNode::Value(Literal(t)) => {
                if let Symbol(name) = t.value() {
                    if !sym_table.get(name.as_str(), Some(scope_ids)).is_some() {
                        return Err((t.line(), format!("Use of undefined variable: {}.", name)));
                    }
                }

                Ok(ast.clone())
            }
            ASTNode::Value(Lambda(args, varargs, body, scope)) => {
                let scope = *scope;

                // Register the known variables and vararg.
                for arg in args {
                    sym_table.register(arg.as_str(), Some(scope))
                }

                if varargs.is_some() {
                    sym_table.register(varargs.as_ref().map(|x| x.as_str()).unwrap(), Some(scope));
                }

                // Visit the bodies with the added context of the new scope.
                scope_ids.push(scope);

                let body: Vec<Result<ASTNode, (u32, String)>> = body
                    .iter()
                    .map(|n| self.try_visit_aux(n, sym_table, scope_ids))
                    .collect();

                // Invalidate the scope.
                scope_ids.pop().unwrap();

                let mut new_body = Vec::new();
                for n in body {
                    if n.is_err() {
                        return n;
                    } else {
                        new_body.push(n.unwrap())
                    }
                }

                Ok(ASTNode::Value(Lambda(
                    args.clone(),
                    varargs.clone(),
                    new_body,
                    scope,
                )))
            }
        }
    }
}

impl ASTVisitor<ASTNode> for SymbolValidation {
    fn try_visit(
        &self,
        ast: &ASTNode,
        sym_table: &mut SymbolTable,
    ) -> Result<ASTNode, (u32, String)> {
        self.try_visit_aux(ast, sym_table, &mut Vec::new())
    }
}

#[derive(Clone)]
// TODO(matthew-c21): Auto-generate lambdas for native functions.
pub struct SymbolTable {
    natives: HashMap<String, String>,
    defs: Vec<HashMap<String, String>>,
    factories: HashMap<String, String>,
    gensym: Gensym,
}

impl SymbolTable {
    /// Strictly used for creating new variable names. Does not actually assign them within the
    /// table.
    pub fn generate(&mut self, base_name: &str) -> String {
        self.gensym.gen(base_name, None)
    }

    // Adds a new name to the table, generating a SymbolTableEntry containing the corresponding C
    // variable name.
    fn register(&mut self, name: &str, scope_id: Option<usize>) {
        let scope_id = scope_id.unwrap_or(0);

        while self.defs.len() <= scope_id {
            self.defs.push(HashMap::new());
        }

        let key = name.to_string();
        let c_name = Gensym::convert(&key);

        if self.get(name, Some(&vec![scope_id])).is_none() {
            self.defs[scope_id].insert(key, c_name);
        }
    }

    pub fn contains_fn(&self, name: &str) -> bool {
        self.natives.contains_key(name)
    }

    // TODO(matthew-c21): Test this function.
    /// Finds the C name of a lisp variable given an optional list of scopes in which to search. If
    /// multiple are found, the name from the last scope found is returned. Always searches global
    /// variables, even if not provided.
    pub fn get(&self, name: &str, scope_ids: Option<&Vec<usize>>) -> Option<&String> {
        if self.natives.contains_key(name) {
            return self.natives.get(name);
        }

        let mut scope_ids = if scope_ids.is_some() {
            scope_ids.unwrap().clone()
        } else {
            vec![0]
        };

        if scope_ids.is_empty() {
            scope_ids.push(0);
        } else if !scope_ids.contains(&0) {
            scope_ids.insert(0, 0);
        }

        let mut result = None;

        for id in scope_ids {
            if id < self.defs.len() && self.defs[id].contains_key(name) {
                result = self.defs[id].get(name)
            }
        }

        result
    }

    pub fn get_factory(&self, name: &str) -> &String {
        self.factories.get(name).unwrap()
    }

    pub fn dummy() -> Self {
        Self {
            natives: HashMap::new(),
            defs: Vec::new(),
            factories: HashMap::new(),
            gensym: Gensym::new(),
        }
    }

    fn validate_json(obj: &json::JsonValue) {
        let required_keys = vec!["functions", "variables", "factories"];

        assert!(obj.is_object());

        for key in &required_keys {
            assert!(obj.has_key(key));
        }

        for name in &required_keys {
            assert!(obj[name.to_string()].is_object());

            for (_, value) in obj[name.to_string()].entries() {
                assert!(value.is_string());
            }
        }

        assert!(obj["factories"].has_key("int"));
        assert!(obj["factories"].has_key("float"));
        assert!(obj["factories"].has_key("complex"));
        assert!(obj["factories"].has_key("rational"));
        assert!(obj["factories"].has_key("string"));
        assert!(obj["factories"].has_key("keyword"));
        assert!(obj["factories"].has_key("true"));
        assert!(obj["factories"].has_key("false"));
        assert!(obj["factories"].has_key("lambda"));
    }

    fn json_to_map(obj: &json::JsonValue, name: &str) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::new();

        for (lisp_name, c_name) in obj[name].entries() {
            map.insert(lisp_name.to_string(), c_name.to_string());
        }

        map
    }

    pub fn load(filename: Option<&str>) -> Self {
        let filename = match filename {
            Some(s) => s,
            None => "natives.json",
        };

        let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

        let obj = json::parse(contents.as_str()).unwrap();

        // Ensure that the JSON object is formed properly.
        Self::validate_json(&obj);

        let defs = vec![Self::json_to_map(&obj, "variables")];

        Self {
            defs,
            natives: Self::json_to_map(&obj, "functions"),
            factories: Self::json_to_map(&obj, "factories"),
            gensym: Gensym::new(),
        }
    }
}

#[cfg(test)]
mod test_utils {
    use crate::ast::*;
    use crate::lex::start;
    use crate::parse::parse;

    pub fn force_from(input: &str) -> Vec<ASTNode> {
        let parse_tree = parse(&start(input).unwrap()).unwrap();

        construct_ast(&parse_tree).unwrap()
    }

    pub fn from_line(input: &str) -> Result<ASTNode, (u32, String)> {
        let mut ast = construct_ast(&parse(&start(input).unwrap()).unwrap())?;

        assert_eq!(1, ast.len());

        Ok(ast.remove(0))
    }
}

#[cfg(test)]
mod visitor_tests {
    use crate::ast::test_utils::{force_from, from_line};
    use crate::ast::*;

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

    #[test]
    fn invalid_symbol_in_definition() {
        let ast = force_from("(define hello world)");
        assert_eq!(1, ast.len());

        let sv = SymbolValidation;
        let mut st = SymbolTable::dummy();

        let node = sv.try_visit(&ast[0], &mut st);

        assert!(node.is_err());
    }

    #[test]
    fn symbol_valid_after_definition() {
        let ast = force_from("(define hello :world) hello");
        assert_eq!(2, ast.len());

        let sv = SymbolValidation;
        let mut st = SymbolTable::dummy();

        let node = sv.try_visit(&ast[0], &mut st);

        assert!(node.is_ok());
    }

    #[test]
    fn symbol_valid_after_redefinition() {
        let ast = force_from(
            "(define hello :world) hello\
                   (define hello :goodbye) hello",
        );
        assert_eq!(4, ast.len());

        let sv = SymbolValidation;
        let mut st = SymbolTable::dummy();

        let node = sv.try_visit(&ast[0], &mut st);

        assert!(node.is_ok());
    }

    #[test]
    fn call_expansion_works_on_lambda() {
        let fu = FunctionUnfurl;
        let lambda = from_line("(lambda () (f (y)))").unwrap();
        let expansion = fu.try_visit(&lambda, &mut SymbolTable::dummy()).unwrap();

        assert_eq!(1, expansion.len());

        if let Lambda(_, _, body, _) = &expansion[0].as_value() {
            assert_eq!(2, body.len());
        } else {
            panic!("Inside of lambda not expanded.")
        }
    }

    #[test]
    fn conditional_expansion_works_on_lambda() {
        let ce = ConditionUnroll;
        let lambda = from_line("(lambda () (if (if x a b) x y))").unwrap();
        let expansion = ce.try_visit(&lambda, &mut SymbolTable::dummy()).unwrap();

        assert_eq!(1, expansion.len());

        if let Lambda(_, _, body, _) = &expansion[0].as_value() {
            // 2 declarations, 2 conditions, and the output value.
            assert_eq!(5, body.len());
        } else {
            panic!("Inside of lambda not expanded.")
        }
    }

    #[test]
    fn symbol_validation_in_lambda() {
        let s = SymbolValidation;

        let ast = from_line("(lambda (x) x)").unwrap();
        s.visit(&ast, &mut SymbolTable::dummy());

        let mut t = SymbolTable::dummy();
        let ast = force_from("(define x 10) (lambda () x)");
        s.visit(&ast[0], &mut t);
        s.visit(&ast[1], &mut t);
    }

    #[test]
    fn symbols_invalid_outside_lambda() {
        let s = SymbolValidation;
        let mut t = SymbolTable::dummy();

        let ast = force_from("(lambda (x) x) x");

        s.visit(&ast[0], &mut t);

        assert!(s.try_visit(&ast[1], &mut t).is_err());
    }

    #[test]
    fn undeclared_var_in_lambda() {
        let s = SymbolValidation;

        let ast = from_line("(lambda (a b) c)").unwrap();
        assert!(s.try_visit(&ast, &mut SymbolTable::dummy()).is_err())
    }

    #[test]
    fn function_unroll_in_condition() {
        let ast = force_from("(if (x (y)) a b)");
        assert_eq!(1, ast.len());

        let u = FunctionUnfurl;
        let ast: Vec<ASTNode> = ast
            .iter()
            .map(|n| u.visit(n, &mut SymbolTable::dummy()))
            .flatten()
            .collect();

        assert_eq!(2, ast.len());
    }
}

#[cfg(test)]
mod ast_tests {
    use crate::ast::test_utils::*;
    use crate::ast::*;

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
                Literal(t) => assert_eq!(Str("foo bar".to_string()), t.value()),
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
    fn wrong_number_condition() {
        let result: Result<ASTNode, (u32, String)> = from_line("(if a b 1 2)");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected exactly 3 arguments in `if` special form. Found 4.",
                msg.as_str()
            )
        }

        let result: Result<ASTNode, (u32, String)> = from_line("(if a b)");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected exactly 3 arguments in `if` special form. Found 2.",
                msg.as_str()
            )
        }

        let result: Result<ASTNode, (u32, String)> = from_line("(if a)");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected exactly 3 arguments in `if` special form. Found 1.",
                msg.as_str()
            )
        }

        let result: Result<ASTNode, (u32, String)> = from_line("(if)");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected exactly 3 arguments in `if` special form. Found 0.",
                msg.as_str()
            )
        }
    }

    #[test]
    fn conditional_non_values() {
        let result: Result<ASTNode, (u32, String)> = from_line("(if #t (define a 1) (define b 2))");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected values for condition, true, and false branches of conditional expression.",
                msg.as_str()
            )
        }

        let result: Result<ASTNode, (u32, String)> = from_line("(if #t (define a 1) b)");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected values for condition, true, and false branches of conditional expression.",
                msg.as_str()
            )
        }

        let result: Result<ASTNode, (u32, String)> = from_line("(if #t a (define b 2))");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected values for condition, true, and false branches of conditional expression.",
                msg.as_str()
            )
        }

        let result: Result<ASTNode, (u32, String)> = from_line("(if (define t #t) a b)");

        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected values for condition, true, and false branches of conditional expression.",
                msg.as_str()
            )
        }
    }

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
    fn from_lambda() {
        let ast = force_from("(lambda (x y) (+ x y))");
        assert_eq!(1, ast.len());

        match &ast[0] {
            ASTNode::Value(Lambda(args, None, body, 1)) => {
                assert_eq!(2, args.len());
                assert_eq!("x", args[0].as_str());
                assert_eq!("y", args[1].as_str());

                if let ASTNode::Value(Call(name, args)) = &body[0] {
                    assert_eq!("+", name.as_str());

                    if let (Literal(t1), Literal(t2)) = (&args[0], &args[1]) {
                        if let (Symbol(n1), Symbol(n2)) = (t1.value(), t2.value()) {
                            assert_eq!("x", n1.as_str());
                            assert_eq!("y", n2.as_str());
                        } else {
                            panic!()
                        }
                    } else {
                        panic!()
                    }
                } else {
                    panic!()
                }
            }
            _ => panic!(),
        }
    }

    #[test]
    fn from_lambdas() {
        let lambdas = force_from("(lambda () nil) (lambda () nil) (lambda() nil)");
        assert_eq!(3, lambdas.len());

        for (i, lambda) in lambdas.iter().enumerate() {
            if let ASTNode::Value(Lambda(_, _, _, j)) = lambda {
                assert_eq!(i + 1, *j);
            } else {
                panic!()
            }
        }
    }

    #[test]
    fn varargs_lambda() {
        let ast = force_from("(lambda (. zs) zs)");
        assert_eq!(1, ast.len());

        match &ast[0] {
            ASTNode::Value(Lambda(args, Some(v), _, 1)) => {
                assert!(args.is_empty());
                assert_eq!("zs", v.as_str());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn empty_args_lambda() {
        let ast = force_from("(lambda () :empty)");
        assert_eq!(1, ast.len());

        match &ast[0] {
            ASTNode::Value(Lambda(args, None, body, 1)) if args.is_empty() => match &body[0] {
                ASTNode::Value(Literal(t)) => assert_eq!(Keyword("empty".to_string()), t.value()),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn malformed_lambda() {
        let result = from_line("(lambda)");
        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected exactly 2 arguments in `lambda` special form. Found 0.",
                msg.as_str()
            );
        } else {
            panic!()
        }

        let result = from_line("(lambda (a b c))");
        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected exactly 2 arguments in `lambda` special form. Found 1.",
                msg.as_str()
            );
        } else {
            panic!()
        }

        let result = from_line("(lambda (a b c) + (a b c))");
        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected exactly 2 arguments in `lambda` special form. Found 3.",
                msg.as_str()
            );
        } else {
            panic!()
        }
    }

    #[test]
    fn no_arg_lambda() {
        let result = from_line("(lambda :not-args :error)");
        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected arglist in second position of `lambda` special form.",
                msg.as_str()
            )
        } else {
            panic!()
        }
    }

    #[test]
    fn non_valued_body_lambda() {
        let result = from_line("(lambda () (define t #t))");
        assert!(result.is_err());

        if let Err((_, msg)) = result {
            assert_eq!(
                "Expected final argument to `lambda` special form to be a value.",
                msg.as_str()
            )
        } else {
            panic!()
        }
    }

    #[test]
    fn no_cons_in_define() {
        assert!(from_line("(define x . 10)").is_err());
        assert!(from_line("(define x 10 . 11)").is_err());
    }

    #[test]
    fn no_cons_in_if() {
        assert!(from_line("(if #t 1 2 . 3)").is_err());
        assert!(from_line("(if #f 1 . 2)").is_err());
    }

    #[test]
    fn no_cons_in_call() {
        assert!(from_line("(format #f . #t)").is_err());
        assert!(from_line("(call f a . b)").is_err());
    }

    #[test]
    fn no_cons_in_lambda() {
        assert!(from_line("(lambda () . 1)").is_err());
        assert!(from_line("(lambda (a b) a . b)").is_err());
    }
}
