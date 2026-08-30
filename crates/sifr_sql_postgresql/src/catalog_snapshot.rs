use serde::{Deserialize, Serialize};
use sifr_sql_contract::{SchemaDocument, SchemaDocumentKind, SchemaObject};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogSnapshot {
    pub document: String,
    pub objects: Vec<SchemaObject>,
}

impl CatalogSnapshot {
    #[must_use]
    pub fn into_document(self) -> SchemaDocument {
        SchemaDocument {
            kind: SchemaDocumentKind::ProviderMetadata,
            document: self.document,
            objects: self.objects,
        }
    }
}
