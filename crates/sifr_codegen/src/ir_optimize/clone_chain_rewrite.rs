use crate::{RustExpr, RustLiteral};

pub(super) fn take_compounded_method_clone(
    receiver: &mut Box<RustExpr>,
    method: &str,
    args: &[RustExpr],
) -> Option<RustExpr> {
    if method != "clone" || !args.is_empty() {
        return None;
    }
    let compounded = matches!(receiver.as_ref(), RustExpr::Clone(_))
        || matches!(
            receiver.as_ref(),
            RustExpr::MethodCall {
                method: inner_method,
                args: inner_args,
                ..
            } if inner_method == "clone" && inner_args.is_empty()
        );
    compounded.then(|| *std::mem::replace(receiver, Box::new(RustExpr::Literal(RustLiteral::Unit))))
}
