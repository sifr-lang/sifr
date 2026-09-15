use super::Demand;
use sifr_ir::HirModule;
use sifr_type_system::{FunctionType, Type};

impl Demand<'_> {
    pub(super) fn ty(&mut self, module_name: &str, module: &HirModule, ty: &Type) {
        match ty {
            Type::Class {
                identity,
                name,
                type_args,
                fields,
                methods,
                parent_class,
            } => {
                self.local_symbol(module_name, module, identity.as_deref().unwrap_or(name));
                for ty in type_args {
                    self.ty(module_name, module, ty);
                }
                for (_, ty) in fields {
                    self.ty(module_name, module, ty);
                }
                for (_, signature) in methods {
                    self.signature(module_name, module, signature);
                }
                if let Some(parent) = parent_class {
                    self.local_symbol(module_name, module, parent);
                }
            }
            Type::Protocol {
                identity,
                name,
                methods,
            } => {
                self.local_symbol(module_name, module, identity.as_deref().unwrap_or(name));
                for (_, signature) in methods {
                    self.signature(module_name, module, signature);
                }
            }
            Type::Enum { identity, name, .. } => {
                self.local_symbol(module_name, module, identity.as_deref().unwrap_or(name));
            }
            Type::Newtype {
                identity,
                name,
                inner,
            } => {
                self.local_symbol(module_name, module, identity.as_deref().unwrap_or(name));
                self.ty(module_name, module, inner);
            }
            Type::Function(signature) | Type::AsyncFunction(signature) => {
                self.signature(module_name, module, signature);
            }
            Type::Callable(params, _, result) | Type::AsyncCallable(params, _, result) => {
                for ty in params {
                    self.ty(module_name, module, ty);
                }
                self.ty(module_name, module, result);
            }
            Type::StructuralRecord(record) => {
                for field in record.fields() {
                    self.ty(module_name, module, field.ty());
                }
            }
            Type::Alias {
                body, type_args, ..
            } => {
                self.ty(module_name, module, body);
                for ty in type_args {
                    self.ty(module_name, module, ty);
                }
            }
            Type::List(ty)
            | Type::Set(ty)
            | Type::Iterable(ty)
            | Type::Iterator(ty)
            | Type::Failure(ty)
            | Type::TimeoutResult(ty)
            | Type::Awaitable(ty)
            | Type::PythonBuffer(ty)
            | Type::PythonDlpackTensor(ty) => self.ty(module_name, module, ty),
            Type::Dict(left, right)
            | Type::Result(left, right)
            | Type::Coroutine(left, right)
            | Type::Task(left, right)
            | Type::TaskResult(left, right)
            | Type::Select2(left, right)
            | Type::BlockingTask(left, right)
            | Type::JoinSet(left, right)
            | Type::AsyncIterator(left, right)
            | Type::AsyncGenerator(left, right) => {
                self.ty(module_name, module, left);
                self.ty(module_name, module, right);
            }
            Type::Tuple(types)
            | Type::Template(types)
            | Type::Union(types)
            | Type::Intersection(types) => {
                for ty in types {
                    self.ty(module_name, module, ty);
                }
            }
            Type::Int
            | Type::FixedInt(_)
            | Type::Float
            | Type::Bool
            | Type::Str
            | Type::Bytes
            | Type::None
            | Type::Range
            | Type::Any
            | Type::Never
            | Type::Unknown
            | Type::TypeVar(_)
            | Type::LiteralInt(_)
            | Type::LiteralStr(_)
            | Type::LiteralBool(_)
            | Type::Decimal
            | Type::BigDecimal
            | Type::PythonArrow(_)
            | Type::PythonDlpackStream => {}
        }
    }

    fn signature(&mut self, name: &str, module: &HirModule, signature: &FunctionType) {
        for (_, ty, _) in &signature.params {
            self.ty(name, module, ty);
        }
        self.ty(name, module, &signature.return_type);
    }
}
