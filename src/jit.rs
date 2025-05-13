use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi, ExecutableBuffer, x64::Assembler};
use crate::parser::*;

pub struct JIT {
    pub code: ExecutableBuffer,
    pub start: dynasmrt::AssemblyOffset,
}

impl JIT {
    pub fn compile(expr: &Expr) -> Self {
        let mut ops = Assembler::new().unwrap();

        let start = ops.offset();
        compile_expr(&mut ops, expr);

        dynasm!(
            ops
            ; .arch x64
            ; ret
        );

        let code = ops.finalize().unwrap();

        JIT { code, start }
    }

    pub fn run(&self) -> Result<(), &str> {
        let func: extern "C" fn() -> i64 = unsafe {
            std::mem::transmute(self.code.ptr(self.start))
        };
        let ret = func();
        if ret == 0 {
            Ok(())
        } else {
            Err("bad :(")
        }
    }
}

fn compile_expr(ops: &mut Assembler, expr: &Expr) {
    match expr {
        Expr::Lit { v } => match v {
            Value::IntV(i) => {
                dynasm!(
                    ops 
                    ; .arch x64
                    ; mov rax, QWORD *i
                );
            }
            Value::BoolV(b) => {
                dynasm!(
                    ops 
                    ; .arch x64
                    ; mov rax, QWORD if *b { 1 } else { 0 }
                );
            }
            _ => todo!(),
        }

        Expr::PrimN { prim, args } => match (prim, args.as_slice()) {
            (Primitive::Add1, [arg]) => {
                compile_expr(ops, arg);
                dynasm!(
                    ops 
                    ; .arch x64
                    ; add rax, 1
                );
            }
            (Primitive::Sub1, [arg]) => {
                compile_expr(ops, arg);
                dynasm!(
                    ops 
                    ; .arch x64
                    ; sub rax, 1
                );
            }
            _ => todo!(),
        }

        Expr::If { if_, then_, else_ } => {
            compile_expr(ops, if_);

            let else_label = ops.new_dynamic_label();
            let end_label = ops.new_dynamic_label();

            dynasm!(
                ops
                ; .arch x64
                ; cmp rax, 0
                ; je =>else_label
            );

            compile_expr(ops, then_);
            dynasm!(
                ops 
                ; .arch x64
                ; jmp =>end_label
            );

            ops.dynamic_label(else_label);
            compile_expr(ops, else_);
            ops.dynamic_label(end_label);
        }

        _ => panic!("Unsupported expression"),
    }
}
