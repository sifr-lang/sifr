mod bytes;
mod collections;
mod encoding;
mod file_handles;
mod open_text_handles;
mod requirements;
mod task;
mod test;

use crate::RustExpr;
use sifr_ir::CompilerIntrinsicId;
use sifr_stdlib_manifest::StdlibFeature;

pub(crate) use collections::{
    is_collection_defaultdict_storage_alias, is_defaultdict_storage_alias,
};
pub(crate) use requirements::additional_required_features;

pub(crate) struct LoweredIntrinsic {
    pub(crate) expr: RustExpr,
    pub(crate) required_feature: Option<StdlibFeature>,
    pub(crate) additional_required_features: &'static [StdlibFeature],
}

pub(crate) fn lower_intrinsic(
    intrinsic: CompilerIntrinsicId,
    args: &[RustExpr],
) -> Option<LoweredIntrinsic> {
    let (expr, required_feature) = match intrinsic {
        CompilerIntrinsicId::OpenBinary => (file_handles::lower_builtin_open(args), None),
        CompilerIntrinsicId::OpenText => (open_text_handles::lower_builtin_open_text(args), None),
        CompilerIntrinsicId::TestAssertEqual => (test::lower_assert_eq(args), None),
        CompilerIntrinsicId::TestAssertNotEqual => (test::lower_assert_ne(args), None),
        CompilerIntrinsicId::TestAssertTrue => (test::lower_assert_true(args), None),
        CompilerIntrinsicId::TestAssertFalse => (test::lower_assert_false(args), None),
        CompilerIntrinsicId::TestAssertAlmostEqual => (test::lower_assert_almost_eq(args), None),
        CompilerIntrinsicId::TestAssertGreaterThan => (test::lower_assert_gt(args), None),
        CompilerIntrinsicId::TestAssertLessThan => (test::lower_assert_lt(args), None),
        CompilerIntrinsicId::StringEncode => (
            encoding::lower_str_encode_result(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        CompilerIntrinsicId::StringEncodeWithEncoding => (
            encoding::lower_str_encode_result(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        CompilerIntrinsicId::BytesDecode => (
            encoding::lower_bytes_decode_result(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        CompilerIntrinsicId::BytesDecodeWithEncoding => (
            encoding::lower_bytes_decode_result(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        CompilerIntrinsicId::BytesFromHex => (bytes::lower_bytes_from_hex(args), None),
        CompilerIntrinsicId::BytesWithSize => (bytes::lower_bytes_with_size(args), None),
        CompilerIntrinsicId::BytesFromIntegers => (bytes::lower_bytes_from_ints(args), None),
        CompilerIntrinsicId::TaskCurrentContext => (
            task::lower_task_current_context(args),
            Some(StdlibFeature::Tokio),
        ),
    };

    Some(LoweredIntrinsic {
        expr: expr?,
        required_feature,
        additional_required_features: additional_required_features(intrinsic),
    })
}
