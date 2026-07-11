use sifr_ir::CompilerIntrinsicId;
use sifr_stdlib_manifest::StdlibFeature;

pub(crate) fn additional_required_features(
    intrinsic: CompilerIntrinsicId,
) -> &'static [StdlibFeature] {
    match intrinsic {
        CompilerIntrinsicId::StringEncode
        | CompilerIntrinsicId::StringEncodeWithEncoding
        | CompilerIntrinsicId::BytesDecode
        | CompilerIntrinsicId::BytesDecodeWithEncoding => &[StdlibFeature::EncodingRs],
        _ => &[],
    }
}
