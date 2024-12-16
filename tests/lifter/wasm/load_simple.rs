use aarch64_air_lifter::arm64::AArch64Lifter;
use std::collections::HashMap;
use std::io::Cursor;
use tnj::arch::reg::Register;
use tnj::pcc::{Constraint, InstConstraint};
use tnj::sym::ExprPool;
use tnj::types::cmp::CmpTy;
use tnj::types::I64;

#[test]
fn from_wasm() {
    let bytes = [
        0x5f, 0x23, 0x3, 0xd5, 0xfd, 0x7b, 0xbf, 0xa9, 0xfd, 0x3, 0x0, 0x91, 0x45, 0x30, 0x40,
        0xf9, 0xa2, 0x48, 0x64, 0xb8, 0xfd, 0x7b, 0xc1, 0xa8, 0xdf, 0x23, 0x3, 0xd5, 0xc0, 0x3,
        0x5f, 0xd6,
    ];

    let mut cursor = Cursor::new(Vec::new());

    let mut exprs = ExprPool::new();

    let heap_base = exprs.val("heap_base", I64);
    let vmctx = exprs.val("vmctx", I64);
    let x2_val = exprs.val(0, I64);
    let x2_val_is_vmctx = exprs.cmp_expr(vmctx, x2_val, CmpTy::Eq);
    let x2 = Register::new("x2", I64);
    let x5 = Register::new("x5", I64);

    let constraints = HashMap::from([(
        0xc,
        vec![
            (
                x5,
                InstConstraint::new(vec![], vec![Constraint::Def(heap_base)]),
            ),
            (
                x2,
                InstConstraint::new(
                    vec![
                        Constraint::Def(x2_val),
                        Constraint::Assertion(x2_val_is_vmctx),
                    ],
                    vec![],
                ),
            ),
        ],
    )]);

    let lifter = AArch64Lifter;
    lifter
        .disassemble_with_constraints(&mut cursor, &bytes, &constraints, &exprs)
        .unwrap();

    let s = String::from_utf8(cursor.into_inner()).expect("Valid UTF-8");

    assert_eq!(
        s,
        r#"0x0000:	hint #0x1a
0x0004:	stp x29, x30, [sp, #-0x10]!
0x0008:	mov x29, sp
← define reg(x2):i64 = @0:i64
← assert vmctx:i64 == @0:i64
0x000c:	ldr x5, [x2, #0x60]
→ define reg(x5):i64 = heap_base:i64
0x0010:	ldr w2, [x5, w4, uxtw]
0x0014:	ldp x29, x30, [sp], #0x10
0x0018:	hint #0x1e
0x001c:	ret
"#
    );
}
