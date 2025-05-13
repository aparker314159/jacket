use std::io;
use std::io::Read;

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

    pub fn run(&self) {
        let func: extern "C" fn() -> i64 = unsafe {
            std::mem::transmute(self.code.ptr(self.start))
        };
        let ret = func();
        println!("function result: {}", ret);
    }
}

macro_rules! call_extern {
    ($ops:ident, $addr:expr) => {dynasm!($ops
        ; .arch x64
        ; mov r15, rsp
        ; and r15, 0b111
        ; sub rsp, r15
        ; mov r8, QWORD $addr as _
        ; call r8
        ; add rsp, r15
    );};
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

            (Primitive::IsZero, [arg]) => {
                compile_expr(ops, arg);
                dynasm!(
                    ops
                    ; .arch x64
                    ; xor r9, r9
                    ; cmp rax, 0
                    ; mov r8, 1
                    ; cmove r9, r8
                    ; mov rax, r9
                );
            }

            (Primitive::ReadByte, _) => {
                // no arity checking yet but we don't compile the arguments
                dynasm!(
                    ops
                    ; .arch x64
                    ;; call_extern!(ops, readbyte)
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

pub extern "C" fn readbyte() -> u8 {
    let mut buf: [u8; 1] = [0];
    io::stdin().read(&mut buf).unwrap();
    buf[0]
}
