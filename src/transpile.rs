use crate::ast::{ASTNode, Statement::*, SymbolTable, Value::*};
use crate::lex::{TokenValue, TokenValue::Symbol};

type LambdaDefinition = (Vec<String>, Option<String>, Vec<ASTNode>, usize);

pub struct Transpiler {
    sym_table: SymbolTable,
    functions: Vec<LambdaDefinition>,
}

impl Transpiler {
    fn imports() -> &'static str {
        "#include \"lisp.h\"\n"
    }

    fn main_definition() -> &'static str {
        "int main(void) {\n"
    }

    fn postfix() -> &'static str {
        "\nreturn 0;\n}"
    }

    /// Find captured variables inside the body of a lambda expression. Assumes all variables within
    /// the body are already valid. Returns a list of captured Lisp symbol names.
    fn find_captures(
        args: &Vec<String>,
        vararg: &Option<String>,
        body: &Vec<ASTNode>,
        scope_id: usize,
    ) -> Vec<String> {
        let mut captures = Vec::new();

        for line in body {
            match line {
                ASTNode::Statement(ExpandedCondition(c, t, f)) => {
                    captures.append(&mut Self::find_captures(
                        args,
                        vararg,
                        &vec![ASTNode::Value(c.clone())],
                        scope_id,
                    ));
                    captures.append(&mut Self::find_captures(args, vararg, t, scope_id));
                    captures.append(&mut Self::find_captures(args, vararg, f, scope_id));
                }
                ASTNode::Value(Call(_, params)) => params.iter().for_each(|v| {
                    captures.append(&mut Self::find_captures(
                        args,
                        vararg,
                        &vec![ASTNode::Value(v.clone())],
                        scope_id,
                    ))
                }),
                ASTNode::Value(Lambda(a, v, b, s)) => {
                    captures.append(&mut Self::find_captures(a, v, b, *s))
                }
                ASTNode::Value(Literal(t)) => {
                    if let Symbol(s) = t.value() {
                        if !(args.contains(&s) || match vararg {
                            Some(y) => y.eq(&s),
                            None => false,
                        }) {
                            // This is a hotfix to avoid capturing generated symbols.
                            if !s.starts_with("gensym") {
                                captures.push(s.clone());
                            }
                        }
                    }
                }
                ASTNode::Statement(Definition(_, v)) => captures.append(&mut Self::find_captures(
                    args,
                    vararg,
                    &vec![ASTNode::Value(v.clone())],
                    scope_id,
                )),
                ASTNode::Statement(Redefinition(_, v)) => {
                    captures.append(&mut Self::find_captures(
                        args,
                        vararg,
                        &vec![ASTNode::Value(v.clone())],
                        scope_id,
                    ))
                }
                ASTNode::Value(Condition(..)) => {
                    panic!("Contact the developer.")
                }
                ASTNode::Statement(Declaration(..)) => (),
            }
        }

        captures
    }

    fn extract_lambda_definitions(ast: &Vec<ASTNode>) -> Vec<LambdaDefinition> {
        let mut output = Vec::new();

        for line in ast {
            match line {
                ASTNode::Value(Condition(..)) => panic!(),
                ASTNode::Value(Call(_, args)) => {
                    for arg in args {
                        output.append(&mut Self::extract_lambda_definitions(&vec![
                            ASTNode::Value(arg.clone()),
                        ]));
                    }
                }
                ASTNode::Value(Lambda(args, vararg, body, scope_id)) => {
                    output.push((args.clone(), vararg.clone(), body.clone(), *scope_id));

                    output.append(&mut Self::extract_lambda_definitions(body));
                }
                ASTNode::Statement(Definition(_, value)) | ASTNode::Statement(Redefinition(_, value)) => {
                    output.append(&mut Self::extract_lambda_definitions(&vec![
                        ASTNode::Value(value.clone()),
                    ]))
                }
                ASTNode::Statement(ExpandedCondition(c, t, f)) => {
                    output.append(&mut Self::extract_lambda_definitions(&vec![
                        ASTNode::Value(c.clone()),
                    ]));
                    output.append(&mut Self::extract_lambda_definitions(t));
                    output.append(&mut Self::extract_lambda_definitions(f));
                }
                ASTNode::Statement(Declaration(..)) | ASTNode::Value(Literal(..)) => (),
            }
        }

        output
    }

    fn lambda_name(&mut self, scope_id: usize) -> String {
        if scope_id > self.functions.len() {
            panic!("Attempt to generate a name for a lambda that has not been extracted.");
        }

        format!("lambda{}_definition", scope_id)
    }

    /// Creates a generated function for a lambda. Arguments are given in the order
    /// (captures, lambda params, varargs).
    fn translate_lambda(&mut self, scope_id: usize) -> String {
        let fn_name = self.lambda_name(scope_id);
        let mut output = format!(
            "struct LispDatum* {}(struct LispDatum** _args, uint32_t _nargs){{",
            fn_name
        );

        let (args, vararg, body, id) = self.functions[scope_id-1].clone();
        let captures = Self::find_captures(&args, &vararg, &body, id);

        let n_captures = captures.len();
        let n_named_args = args.len();

        for (i, capture) in captures.iter().enumerate() {
            // TODO(matthew-c21): At some point, every individual symbol will need to have an
            //  associated scope. However, since only lambdas have a scope and all other variables
            //  must be global, we can rough out the idea here pretty easily. The alternative is
            //  going through things like this and just renaming all captured variables before
            //  reaching the translation stage.
            output.push_str(
                format!(
                    "struct LispDatum* {} = _args[{}];",
                    self.sym_table.get(capture.as_str(), Some(&vec![id])).unwrap(),
                    i
                )
                .as_str(),
            )
        }

        for (i, arg) in args.iter().enumerate() {
            output.push_str(
                format!(
                    "struct LispDatum* {} = _args[{}];",
                    self.sym_table
                        .get(arg.as_str(), Some(&vec![scope_id]))
                        .unwrap(),
                    n_captures + i
                )
                .as_str(),
            )
        }

        if vararg.is_some() {
            output.push_str(
                format!(
                    "struct LispDatum* {} = {}(_args + (_nargs - {}), {});",
                    self.sym_table.get(vararg.as_ref().unwrap().as_str(), Some(&vec![scope_id])).unwrap(),
                    self.sym_table.get("list", None).unwrap(),
                    n_captures + n_named_args,
                    n_captures + n_named_args
                )
                .as_str(),
            )
        }

        let mut lines: Vec<String> = body.iter().map(|n| self.translate_node(n, &mut vec![scope_id])).flatten().collect();
        let ret_value = lines.pop().unwrap();

        for line in &lines {
            output.push_str(line.as_str());

            if !(output.ends_with(';') || output.ends_with('}')) {
                output.push(';');
            }
        }

        output.push_str(format!("return {};}}", ret_value).as_str());

        output
    }

    /// Create a new Transpiler.
    ///
    /// @param factories: HashMap containing at least the keys: int, float, complex, rational,
    ///     keyword, string, true, false, and associates them with the C runtime factory functions.
    pub fn new(sym_table: SymbolTable) -> Self {
        Self {
            sym_table,
            functions: Vec::new(),
        }
    }

    /// Convert a modified AST to a String. If the AST is not in a reduced form as a result of
    /// visitors in the ast module, this program may crash. Specifically, it assumes that all
    /// functions, conditions, and lambdas have been unrolled.
    pub fn translate(&mut self, ast: &Vec<ASTNode>) -> String {
        let mut output = String::from(Self::imports());
        let mut scope_ids = Vec::new();

        self.functions.clear();
        self.functions
            .append(&mut Self::extract_lambda_definitions(ast));

        for (_, _, _, scope_id) in self.functions.clone() {
            output.push_str(self.translate_lambda(scope_id).as_str())
        }

        output.push_str(Self::main_definition());

        for node in ast {
            for line in self.translate_node(node, &mut scope_ids) {
                output.push_str(line.as_str());

                // This will make extraneous semicolons for forms that generate braces, such as
                // conditionals. However, this is essentially harmless, and can safely be ignored.
                if !(output.ends_with(';') || output.ends_with('}')) {
                    output.push(';');
                }
            }
        }

        output.push_str(Self::postfix());

        output
    }

    fn translate_node(&mut self, node: &ASTNode, scope_ids: &mut Vec<usize>) -> Vec<String> {
        let mut output = Vec::new();

        match node {
            ASTNode::Value(Literal(t)) => {
                match t.value() {
                    TokenValue::Int(x) => {
                        output.push(format!("{}({})", self.sym_table.get_factory("int"), x))
                    }
                    TokenValue::Float(x) => {
                        output.push(format!("{}({})", self.sym_table.get_factory("float"), x))
                    }
                    TokenValue::Complex(x, y) => {
                        output.push(format!("{}({},{})", self.sym_table.get_factory("complex"), x, y))
                    }
                    TokenValue::Rational(x, y) => {
                        output.push(format!("{}({},{})", self.sym_table.get_factory("rational"), x, y))
                    }
                    TokenValue::Str(x) => {
                        output.push(format!("{}({:?})", self.sym_table.get_factory("string"), x))
                    }
                    TokenValue::Keyword(x) => {
                        output.push(format!("{}(\"{}\")", self.sym_table.get_factory("keyword"), x))
                    }
                    TokenValue::True => {
                        output.push(format!("{}()", self.sym_table.get_factory("true")))
                    }
                    TokenValue::False => {
                        output.push(format!("{}()", self.sym_table.get_factory("false")))
                    }
                    TokenValue::Symbol(s) => {
                        output.push(self.sym_table.get(s.as_str(), Some(scope_ids)).unwrap().clone())
                    }
                    TokenValue::Nil => {
                        output.push(format!("{}()", self.sym_table.get_factory("nil")))
                    }
                    _ => panic!("Encountered invalid token literal in AST. Contact the developer.")
                }
            }
            ASTNode::Value(Call(callee, args)) => {
                let arglist = self.sym_table.generate("arglist");

                output.push(format!("struct LispDatum* {}[{}];\n", arglist, args.len()));

                for (i, arg) in args.iter().enumerate() {
                    let mut prefix = self.translate_node(&ASTNode::Value(arg.clone()), scope_ids);
                    let line = format!("{}[{}] = {};\n", arglist, i, prefix.pop().unwrap());
                    output.append(&mut prefix);
                    output.push(line);
                }

                output.push(format!("{}({}, {})", self.sym_table.get(callee.as_str(), Some(scope_ids)).unwrap(), arglist, args.len()))
            }
            ASTNode::Value(Condition(..)) => {
                panic!("Conditions should have been upgraded to expanded conditions before this step. Contact the developer.")
            }
            ASTNode::Value(Lambda(args, vararg, body, scope_id)) => {
                let lambda_fn_name = self.lambda_name(*scope_id);
                let capture_vec_name = self.sym_table.generate("lambda_captures");
                let captures = Self::find_captures(args, vararg, body, *scope_id);

                let capture_vec_name = if !captures.is_empty() {
                    capture_vec_name
                } else {
                    "NULL".to_string()
                };

                scope_ids.push(*scope_id);

                if !captures.is_empty() {
                    output.push(format!("struct LispDatum* {}[{}];", capture_vec_name, captures.len()));

                    for (i, capture) in captures.iter().enumerate() {
                        output.push(format!("{}[{}] = {};", capture_vec_name, i, self.sym_table.get(capture, Some(scope_ids)).unwrap()))
                    }
                }

                assert_eq!(*scope_id, scope_ids.pop().unwrap());

                // NOTE(matthew-c21): NULL is used for all non-native lambdas.
                output.push(format!("{}({}, {}, {}, NULL)", self.sym_table.get_factory("lambda"), lambda_fn_name, capture_vec_name, captures.len()));
            }
            ASTNode::Statement(Declaration(name)) => {
                output.push(format!("struct LispDatum* {}", self.sym_table.get(name.as_str(), Some(scope_ids)).unwrap()))
            }
            ASTNode::Statement(Definition(name, value)) => {
                let mut value = self.translate_node(&ASTNode::Value(value.clone()), scope_ids);

                let v = value.pop().unwrap();
                output.append(&mut value);

                output.push(format!("struct LispDatum* {} = {}", self.sym_table.get(name.as_str(), Some(scope_ids)).unwrap(), v))
            }
            ASTNode::Statement(Redefinition(name, value)) => {
                let mut value = self.translate_node(&ASTNode::Value(value.clone()), scope_ids);

                let v = value.pop().unwrap();
                output.append(&mut value);

                output.push(format!("{} = {}", self.sym_table.get(name.as_str(), Some(scope_ids)).unwrap(), v))
            }
            ASTNode::Statement(ExpandedCondition(c, t, f)) => {
                let mut c = self.translate_node(&ASTNode::Value(c.clone()), scope_ids);
                let cond = c.pop().unwrap();
                output.append(&mut c);
                output.push(format!("if (truthy({})) {{", cond));

                for v in t {
                    output.append(&mut self.translate_node(v, scope_ids));
                }

                output.push(String::from("} else {"));

                for v in f {
                    output.append(&mut self.translate_node(v, scope_ids));
                }

                output.push(String::from("}"));
            }
        }

        output
    }
}
