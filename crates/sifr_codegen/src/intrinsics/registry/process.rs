//! Native process intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustStmt};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

pub(crate) fn lower_process_output(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__program".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__args".to_string(),
                ty: None,
                value: arg_expr(args, 1),
            },
            RustStmt::Let {
                mutable: false,
                name: "__cwd".to_string(),
                ty: None,
                value: arg_expr(args, 2),
            },
            RustStmt::Let {
                mutable: false,
                name: "__shell".to_string(),
                ty: None,
                value: arg_expr(args, 3),
            },
        ],
        expr: Some(Box::new(RustExpr::Ident(
            "{\n    let mut __command = if __shell {\n        if cfg!(target_os = \"windows\") {\n            let mut __cmd = std::process::Command::new(\"cmd\");\n            __cmd.arg(\"/C\").arg(&__program).args(&__args);\n            __cmd\n        } else {\n            let mut __cmd = std::process::Command::new(\"sh\");\n            __cmd.arg(\"-c\").arg(&__program).args(&__args);\n            __cmd\n        }\n    } else {\n        let mut __cmd = std::process::Command::new(&__program);\n        __cmd.args(&__args);\n        __cmd\n    };\n    if let Some(__dir) = __cwd {\n        __command.current_dir(__dir);\n    }\n    let __output = __command\n        .output()\n        .map_err(|__err| ProcessError::new(__err.to_string()))?;\n    Ok((\n        __output.stdout,\n        __output.stderr,\n        __output.status.code().unwrap_or(-1) as i64,\n    ))\n}"
                .to_string(),
        ))),
    })
}
