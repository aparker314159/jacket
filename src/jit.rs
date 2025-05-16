use std::io::{self, Write};
use std::io::Read;

use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi, ExecutableBuffer, x64::Assembler};
use crate::parser::*;

use yaxpeax_x86::long_mode::RegSpec;
use once_cell::sync::Lazy;

static regmap: Lazy<Vec<RegSpec>> = Lazy::new(|| vec![RegSpec::rax(), RegSpec::rdi(), RegSpec::rsi(), RegSpec::rbx(), RegSpec::rcx(),
                                                RegSpec::r10(), RegSpec::r11(), RegSpec::r12(), RegSpec::r13()]);

pub struct JIT {
    pub code: ExecutableBuffer,
    pub start: dynasmrt::AssemblyOffset,
}

impl JIT {
    pub fn compile(expr: &Expr) -> Self {
        let mut ops = Assembler::new().unwrap();

        let start = ops.offset();
        compile_expr(&mut ops, expr, 0);

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
        let _ = func();
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

fn compile_expr(ops: &mut Assembler, expr: &Expr, offset: usize) -> usize {
    match expr {
        Expr::Lit { v } => match v {
            Value::IntV(i) => {
                dynasm!(
                    ops 
                    ; .arch x64
                    ; mov rdx, 0b01
                    ; mov Rq(regmap[offset].num()), QWORD *i
                );
                offset
            }
            Value::BoolV(b) => {
                dynasm!(
                    ops 
                    ; .arch x64
                    ; mov rdx, 0b10
                    ; mov Rq(regmap[offset].num()), QWORD if *b { 1 } else { 0 }
                );
                offset
            }
            Value::CharV(c) => {
                dynasm!(
                    ops 
                    ; .arch x64
                    ; mov rdx, 0b11
                    ; mov Rq(regmap[offset].num()), QWORD *c as i64
                );
                offset
            }
            _ => todo!(),
        }

        Expr::PrimN { prim, args } => match (prim, args.as_slice()) {
            (Primitive::Add1, [arg]) => {
                match arg {
                    Expr::Lit { v } => match v {
                        Value::IntV( n ) => {
                            dynasm!(
                                ops
                                ; .arch x64
                                ; mov Rq(regmap[offset].num()), (n + 1).try_into().unwrap()
                            );
                            offset
                        },
                        _ => {
                            dynasm!(
                                ops
                                ; .arch x64
                                ; jmp 0x0
                            );
                            offset
                        }
                    },
                    _ => {
                        let ret = compile_expr(ops, arg, offset);
                        dynasm!(
                            ops 
                            ; .arch x64
                            ; cmp rdx, 0b01
                            ; jne 0x0
                            ; add Rq(regmap[ret].num()), 1
                        );
                        ret
                    }
                }
                
            }

            (Primitive::Sub1, [arg]) => {
                match arg {
                    Expr::Lit { v } => match v {
                        Value::IntV( n ) => {
                            dynasm!(
                                ops
                                ; .arch x64
                                ; mov Rq(regmap[offset].num()), (n - 1).try_into().unwrap()
                            );
                            offset
                        },
                        _ => {
                            dynasm!(
                                ops
                                ; .arch x64
                                ; jmp 0x0
                            );
                            offset
                        }
                    },
                    _ => {
                        let ret = compile_expr(ops, arg, offset);
                        dynasm!(
                            ops 
                            ; .arch x64
                            ; cmp rdx, 0b01
                            ; jne 0x0
                            ; sub Rq(regmap[ret].num()), 1
                        );
                        ret
                    }
                }
            }

            (Primitive::IsZero, [arg]) => {
                let ret = compile_expr(ops, arg, offset);
                dynasm!(
                    ops
                    ; .arch x64
                    ; cmp rdx, 0b01
                    ; jne 0x0
                    ; xor r9, r9
                    ; cmp Rq(regmap[ret].num()), 0
                    ; mov r8, 1
                    ; cmove r9, r8
                    ; mov rax, r9
                );
                ret
            }

            (Primitive::IsChar, [arg]) => {
                let ret = compile_expr(ops, arg, offset);
                dynasm!(
                    ops
                    ; .arch x64
                    ; cmp rdx, 0b11
                    ; xor r9, r9
                    ; mov r8, 1
                    ; cmove r9, r8
                    ; mov Rq(regmap[ret].num()), r9
                );
                ret
            }

            (Primitive::IntToChar, [arg]) => {
                let ret = compile_expr(ops, arg, offset);
                dynasm!(
                    ops
                    ; .arch x64
                    ; mov rdx, 0b11
                );
                ret
            }

            (Primitive::CharToInt, [arg]) => {
                let ret = compile_expr(ops, arg, offset);
                dynasm!(
                    ops
                    ; .arch x64
                    ; mov rdx, 0b01
                );
                ret
            }

            (Primitive::ReadByte, _) => {
                // no arity checking yet but we don't compile the arguments
                dynasm!(
                    ops
                    ; .arch x64
                    ;; call_extern!(ops, readbyte)
                );
                offset
            }

            (Primitive::WriteByte, _) => {
                // no arity checking yet but we don't compile the arguments
                dynasm!(
                    ops
                    ; .arch x64
                    ;; call_extern!(ops, writebyte)
                );
                offset
            }

            _ => todo!(),
        }

        Expr::If { if_, then_, else_ } => {
            let after_if = compile_expr(ops, if_, offset);

            let else_label = ops.new_dynamic_label();
            let end_label = ops.new_dynamic_label();

            dynasm!(
                ops
                ; .arch x64
                ; cmp Rq(regmap[offset].num()), 0
                ; je =>else_label
            );

            let after_then = compile_expr(ops, then_, after_if);
            dynasm!(
                ops 
                ; .arch x64
                ; jmp =>end_label
            );

            ops.dynamic_label(else_label);
            let ret = compile_expr(ops, else_, after_then);
            ops.dynamic_label(end_label);

            ret
        }
        _ => panic!("Unsupported expression"),
    }
}

pub extern "C" fn readbyte() -> u8 {
    let mut buf: [u8; 1] = [0];
    io::stdin().read(&mut buf).unwrap();
    buf[0]
}

pub extern "C" fn writebyte(buf: *const u8) {
    let slice = unsafe { std::slice::from_raw_parts(buf, 1) };
    io::stdout().write_all(slice).unwrap();
}
