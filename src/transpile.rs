use crate::ast::Scope::Global;
use crate::ast::{ASTNode, Statement::*, SymbolTable, Value::*};
use crate::lex::TokenValue;

pub struct Transpiler {
    sym_table: SymbolTable,
}

impl Transpiler {
    fn prefix() -> &'static str {
        "#include \"lisp.h\"\n\
        int main(void) {\n"
    }

    fn postfix() -> &'static str {
        "\nreturn 0;\n}"
    }

    /// Create a new Transpiler.
    ///
    /// @param factories: HashMap containing at least the keys: int, float, complex, rational,
    ///     keyword, string, true, false, and associates them with the C runtime factory functions.
    pub fn new(sym_table: SymbolTable) -> Self {
        Self { sym_table }
    }

    /// Convert a modified AST to a String. If the AST is not in a reduced form as a result of
    /// visitors in the ast module, this program may crash. Specifically, it assumes that all
    /// functions, conditions, and lambdas have been unrolled.
    pub fn translate(&mut self, ast: Vec<ASTNode>) -> String {
        let mut output = String::from(Self::prefix());

        for node in &ast {
            for line in self.translate_node(node) {
                output.push_str(line.as_str());

                // This will make extraneous semicolons for forms that generate braces, such as
                // conditionals. However, this is essentially harmless, and can safely be ignored.
                output.push(';');
            }
        }

        output.push_str(Self::postfix());

        output
    }

    fn translate_node(&mut self, node: &ASTNode) -> Vec<String> {
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
                        output.push(format!("{}({})", self.sym_table.get_factory("keyword"), x))
                    }
                    TokenValue::True => {
                        output.push(format!("{}()", self.sym_table.get_factory("true")))
                    }
                    TokenValue::False => {
                        output.push(format!("{}()", self.sym_table.get_factory("false")))
                    }
                    TokenValue::Symbol(s) => {
                        output.push(self.sym_table.get(s.as_str()).unwrap().clone())
                    }
                    _ => panic!("Encountered invalid token literal in AST. Contact the developer.")
                }
            }
            ASTNode::Value(Call(callee, args)) => {
                 let arglist = self.sym_table.generate("arglist", Global);

                            output.push(format!("struct LispDatum* {}[{}];\n", arglist, args.len()));

                            for (i, arg) in args.iter().enumerate() {
                                let mut prefix = self.translate_node(&ASTNode::Value(arg.clone()));
                                let line = format!("{}[{}] = {};\n", arglist, i, prefix.pop().unwrap());
                                output.append(&mut prefix);
                                output.push(line);
                            }

                            output.push(format!("{}({}, {})", self.sym_table.get(callee.as_str()).unwrap(), arglist, args.len()))

            }
            ASTNode::Value(Condition(c, t, f)) => {
                panic!("Conditions should have been upgraded to expanded conditions before this step. Contact the developer.")
            }
            ASTNode::Value(Lambda(..)) => {
                unimplemented!()
            }
            ASTNode::Statement(Declaration(name)) => {
                output.push(format!("struct LispDatum* {}", self.sym_table.get(name.as_str()).unwrap()))
            }
            ASTNode::Statement(Definition(name, value)) => {
                let mut value = self.translate_node(&ASTNode::Value(value.clone()));

                let v = value.pop().unwrap();
                output.append(&mut value);

                output.push(format!("struct LispDatum* {} = {}", self.sym_table.get(name.as_str()).unwrap(), v))
            }
            ASTNode::Statement(Redefinition(name, value)) => {
                let mut value = self.translate_node(&ASTNode::Value(value.clone()));

                let v = value.pop().unwrap();
                output.append(&mut value);

                output.push(format!("{} = {}", self.sym_table.get(name.as_str()).unwrap(), v))
            }
            ASTNode::Statement(ExpandedCondition(c, t, f)) => {
                let mut c = self.translate_node(&ASTNode::Value(c.clone()));
                let l = c.len();
                output.extend_from_slice(&mut c[..l-1]);
                output.push(format!("if ({}) {{", c.last().unwrap()));

                for v in t {
                    output.append(&mut self.translate_node(v));
                }

                output.push(String::from("} else {"));

                for v in f {
                    output.append(&mut self.translate_node(v));
                }

                output.push(String::from("}"));
            }
        }

        output
    }
}
