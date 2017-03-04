use std::collections::HashMap;
use llvm::*;
use llvm::Attribute::*;
use parser::*;
use parser;
use llvm::Function;


pub fn generate_expression<'a, 'b>(node: &'b Expr,
                               values: &'a HashMap<&String, &'a Arg>,
                               builder: &'a CSemiBox<'a, Builder>,
                               module: &'a CSemiBox<'a, Module>,
                               context: &'a CBox<Context>) -> Result<&'a Value, String> {
    match *node {
        Expr::Number(n) => Ok(n.compile(&context)),
        Expr::Variable(ref v) => Ok(values.get(v).ok_or(
                                format!("There is no variable named {}", v))?
                            ),
        Expr::Binary {op, ref lhs, ref rhs} => {
            let l = generate_expression(&*lhs, &values, &builder, &module, &context)?;
            let r = generate_expression(&*rhs, &values, &builder, &module, &context)?;
            match op {
                '+' => Ok(builder.build_add(&l, &r)),
                '-' => Ok(builder.build_sub(&l, &r)),
                '*' => Ok(builder.build_mul(&l, &r)),
                '<' => {
                    let comp = builder.build_cmp(&l, &r, Predicate::LessThan);
                    let res = builder.build_bit_cast(&comp, &Type::get::<f64>(&context));
                    Ok(res)
                }
                _ => return Err(format!("{} is an invalid operator!", op))
            }
        },
        Expr::Call {ref name, ref args} => {
            let func = module.get_function(name).ok_or(format!("There is no function named {}!", name))?;
            let passed_args = args.len();
            let expected_args = func.get_signature().num_params();
            if expected_args != passed_args {
                return Err(format!("{} takes {} args, but you passed {}!", name, expected_args, passed_args))
            }
            let mut passed = Vec::new();
            for arg in args {
                passed.push(generate_expression(&arg, &values, &builder, &module, &context)?)
            }
            Ok(builder.build_call(&func, &passed))
        }
    }
}
pub fn generate_prototype<'a>(prototype: &Prototype,
                          module: &'a CSemiBox<'a, Module>,
                          context: &'a CBox<Context>) -> Result<&'a Function, String> {
    let arg_types = vec![Type::get::<f64>(&context); prototype.args.len()];
    let sig = FunctionType::new(Type::get::<f64>(&context), &arg_types);
    let func = module.add_function(&prototype.name, sig);
    for arg_index in 0..prototype.args.len() {
        &func[arg_index].set_name(&prototype.args[arg_index]);
    }
    Ok(func)
}
pub fn generate_function<'a>(function_ast: &parser::Function,
                         builder: &'a CSemiBox<'a, Builder>,
                         module: &'a CSemiBox<'a, Module>,
                         context: &'a CBox<Context>) -> Result<&'a Function, String> {
    let mut func = module.get_function(&function_ast.prototype.name);
    let func = if func.is_none() {
        generate_prototype(&function_ast.prototype, &module, &context)?
    } else {
        func.unwrap()
    };
    let block = func.append("entry");
    builder.position_at_end(block);
    let mut values = HashMap::new();
    for (i, name) in function_ast.prototype.args.iter().enumerate() {
        values.insert(name, &func[i]);
    }
    let ret = generate_expression(&function_ast.body, &values,
                                       &builder, &module, &context)?;
    builder.build_ret(ret);
    module.verify().unwrap();
    Ok(func)
}


#[cfg(test)]
mod tests {
    use super::*;
    use parser;
    #[test]
    fn test_codegen() {
        let mut parser = parser::Parser::from_source("def foo(a) a + a");
        let ast = parser.parse_definition().unwrap();
        let ctx = Context::new();
        let builder = Builder::new(&ctx);
        let module = Module::new("test", &ctx);
        let func = generate_function(&ast, &builder, &module, &ctx).unwrap();
        module.write_bitcode("test.bitcode").unwrap();
    }
    #[test]
    fn test_toplevel_codegen() {
        let mut parser = parser::Parser::from_source("1 + 1");
        let ast = parser.parse_top_level_expr().unwrap();
        let ctx = Context::new();
        let builder = Builder::new(&ctx);
        let module = Module::new("test", &ctx);
        let func = generate_function(&ast, &builder, &module, &ctx).unwrap();
        module.write_bitcode("test.bitcode").unwrap();
    }
}
