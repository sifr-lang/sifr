use crate::graph::derive::SifrPackageId;
use crate::manifest::sifr::ImportRoot;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageSourceMap {
    pub roots: BTreeMap<(SifrPackageId, ImportRoot), PathBuf>,
}
