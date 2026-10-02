#![allow(unsafe_code)] // WIT Bindgen generates the component ABI boundary.

use sifr_compiler_component::EmbeddedAnalysisRequest;

wit_bindgen::generate!({
    path: "../sifr_compiler_component/wit",
    world: "embedded-language-provider",
});

struct SqliteComponent;

impl Guest for SqliteComponent {
    fn analyze(request: Vec<u8>) -> Vec<u8> {
        let Ok(request) = serde_json::from_slice::<EmbeddedAnalysisRequest>(&request) else {
            return Vec::new();
        };
        let response = match crate::execute_embedded_request(request.clone()) {
            Ok(mut response) => {
                if sifr_sql_contract::project_provider_diagnostics(&request, &mut response).is_err()
                {
                    return Vec::new();
                }
                response
            }
            Err(diagnostic) => {
                let semantic = diagnostic.semantic();
                let Ok(response) =
                    sifr_sql_contract::provider_diagnostic_response(&request, &semantic)
                else {
                    return Vec::new();
                };
                response
            }
        };
        serde_json::to_vec(&response).unwrap_or_default()
    }
}

export!(SqliteComponent);
