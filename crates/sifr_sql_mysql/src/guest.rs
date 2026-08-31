#![allow(unsafe_code)] // WIT Bindgen generates the component ABI boundary.

use sifr_compiler_component::EmbeddedAnalysisRequest;

wit_bindgen::generate!({
    path: "../sifr_compiler_component/wit",
    world: "embedded-language-provider",
});

struct MysqlComponent;

impl Guest for MysqlComponent {
    fn analyze(request: Vec<u8>) -> Vec<u8> {
        let Ok(request) = serde_json::from_slice::<EmbeddedAnalysisRequest>(&request) else {
            return Vec::new();
        };
        let Ok(response) = crate::execute_embedded_request(request) else {
            return Vec::new();
        };
        serde_json::to_vec(&response).unwrap_or_default()
    }
}

export!(MysqlComponent);
