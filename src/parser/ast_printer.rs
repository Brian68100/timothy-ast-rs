#[derive(PartialEq)]
enum PrinterState {
    Done,
    NotDoneYet,
}

pub(crate) struct AstPrinter;

impl<'a> AstPrinter {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn run(&mut self, root: &mut Option<Box<Ast>>) {
        let ret = self.print(root, 0);
        match ret {
            PrinterState::Done => {}
            PrinterState::NotDoneYet => {
                panic!("\nAST printer is not done even though it should be");
            }
        }
    }

    fn print(&mut self, node: &mut Option<Box<Ast>>, level: usize) -> PrinterState {
        match node {
            Some(ref mut ast) => {
                self.indent(level);
                println!("(Ast Type: {}; Level: {})", ast, level);

                match **ast {
                    Ast::ClosureDecl {
                        ref mut name,
                        ref mut fun_ty,
                        ref mut params,
                        ref mut listing,
                        ..
                    } => {
                        self.indent(level + 1);
                        println!("Name: {}", name);
                        self.indent(level + 1);
                        println!("Type: {}", fun_ty);
                        self.indent(level + 1);
                        println!("Params:");
                        self.indent(level + 1);
                        println!("{{");
                        for i in params {
                            self.print(&mut Some(i.clone()), level + 1);
                        }
                        self.indent(level + 1);
                        println!("}}");
                        self.indent(level + 1);
                        println!("Listing:");
                        self.indent(level + 1);
                        println!("{{");
                        for i in listing {
                            self.print(&mut Some(i.clone()), level + 1);
                        }
                        self.indent(level + 1);
                        println!("}}");
                    }
                    Ast::LetDecl {
                        ref mut name,
                        ref mut expr,
                        ..
                    } => {
                        self.indent(level + 1);
                        println!("Name: {}", name);
                        self.indent(level + 1);
                        println!("Expr:");
                        self.print(&mut Some(expr.clone()), level + 2);
                    }
                    Ast::ExprStmt { ref mut expr, .. } => {
                        self.indent(level + 1);
                        println!("Expr:");

                        self.indent(level + 1);
                        println!("{{");

                        self.print(&mut Some(expr.clone()), level + 1);

                        self.indent(level + 1);
                        println!("}}");
                    }
                    Ast::Block { ref mut listing } => {
                        self.indent(level + 1);
                        println!("Listing:");
                        self.indent(level + 1);
                        println!("{{");
                        for i in listing {
                            self.print(&mut Some(i.clone()), level + 1);
                        }
                        self.indent(level + 1);
                        println!("}}");
                    }
                    Ast::Add(ref mut left, ref mut right)
                    | Ast::Subtract(ref mut left, ref mut right)
                    | Ast::Multiply(ref mut left, ref mut right)
                    | Ast::Divide(ref mut left, ref mut right)
                    | Ast::Modulo(ref mut left, ref mut right)
                    | Ast::Power(ref mut left, ref mut right)
                    | Ast::LAnd(ref mut left, ref mut right)
                    | Ast::LOr(ref mut left, ref mut right)
                    | Ast::And(ref mut left, ref mut right)
                    | Ast::Or(ref mut left, ref mut right)
                    | Ast::XOr(ref mut left, ref mut right)
                    | Ast::LShift(ref mut left, ref mut right)
                    | Ast::RShift(ref mut left, ref mut right)
                    | Ast::Assign(ref mut left, ref mut right)
                    | Ast::AddAssign(ref mut left, ref mut right)
                    | Ast::SubtractAssign(ref mut left, ref mut right)
                    | Ast::MultiplyAssign(ref mut left, ref mut right)
                    | Ast::DivideAssign(ref mut left, ref mut right)
                    | Ast::ModuloAssign(ref mut left, ref mut right)
                    | Ast::PowerAssign(ref mut left, ref mut right)
                    | Ast::AndAssign(ref mut left, ref mut right)
                    | Ast::OrAssign(ref mut left, ref mut right)
                    | Ast::XOrAssign(ref mut left, ref mut right)
                    | Ast::LShiftAssign(ref mut left, ref mut right)
                    | Ast::RShiftAssign(ref mut left, ref mut right) => {
                        self.indent(level + 1);
                        println!("Left:");
                        self.print(&mut left.clone(), level + 1);
                        self.indent(level + 1);
                        println!("Right:");
                        self.print(&mut right.clone(), level + 1);
                    }
                    Ast::Value { ref mut val, .. } => {
                        self.indent(level + 1);
                        println!("Value: {}", val);
                    }
                    Ast::VariableId {
                        ref mut name,
                        ref mut arity,
                        ..
                    } => {
                        self.indent(level + 1);
                        println!("Name: {}", name);
                    }
                    Ast::Call {
                        ref mut recv,
                        ref mut args,
                    } => {
                        self.indent(level + 1);
                        println!("Receiver:");
                        self.print(&mut recv.clone(), level + 1);
                        self.indent(level + 1);
                        println!("Args:");
                        self.indent(level + 1);
                        println!("{{");
                        for i in 0..args.len() {
                            self.print(&mut Some(args[i].clone()), level + 1);
                            if i < args.len() - 1 {
                                self.indent(level + 1);
                                println!(",");
                            }
                        }
                        self.indent(level + 1);
                        println!("}}");
                    }
                    _ => {
                        panic!("unrecognized node type");
                    }
                }
            }
            None => {
                panic!("node doesn't exist");
            }
        }

        if level == 0 {
            PrinterState::Done
        } else {
            PrinterState::NotDoneYet
        }
    }

    fn indent(&mut self, level: usize) {
        for _i in 0..level * 2 {
            print!(" ");
        }
    }
}
