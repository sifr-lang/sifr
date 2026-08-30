use crate::HirExpr;
use sifr_type_system::Type;

impl HirExpr {
    /// Get the type of this expression.
    pub fn ty(&self) -> &Type {
        match self {
            Self::IntLiteral(_) | Self::LargeIntLiteral(_) => &Type::Int,
            Self::FloatLiteral(_) => &Type::Float,
            Self::StringLiteral(_) => &Type::Str,
            Self::BoolLiteral(_) => &Type::Bool,
            Self::NoneLiteral => &Type::None,
            Self::Name { ty, .. }
            | Self::BinOp { ty, .. }
            | Self::UnaryOp { ty, .. }
            | Self::Compare { ty, .. }
            | Self::BoolOp { ty, .. }
            | Self::Call { ty, .. }
            | Self::GenericCall { ty, .. }
            | Self::PythonCall { ty, .. }
            | Self::IntrinsicCall { ty, .. }
            | Self::Await { ty, .. }
            | Self::IteratorCall { ty, .. }
            | Self::IfExpr { ty, .. }
            | Self::RangeLiteral { ty, .. }
            | Self::ListLiteral { ty, .. }
            | Self::SetLiteral { ty, .. }
            | Self::DictLiteral { ty, .. }
            | Self::TupleLiteral { ty, .. }
            | Self::Index { ty, .. }
            | Self::MethodCall { ty, .. }
            | Self::ContainsOp { ty, .. }
            | Self::FString { ty, .. }
            | Self::Slice { ty, .. }
            | Self::WalrusExpr { ty, .. }
            | Self::FieldAccess { ty, .. }
            | Self::ConstructorCall { ty, .. }
            | Self::QuestionMark { ty, .. }
            | Self::OkWrap { ty, .. }
            | Self::ErrWrap { ty, .. }
            | Self::SuperCall { ty, .. }
            | Self::Lambda { ty, .. }
            | Self::ListComp { ty, .. }
            | Self::DictComp { ty, .. }
            | Self::SetComp { ty, .. }
            | Self::GeneratorExpr { ty, .. }
            | Self::EnumVariant { ty, .. } => ty,
            Self::TemplateString(template) => &template.ty,
        }
    }
}
