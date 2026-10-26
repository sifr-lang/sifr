//! Explicit maintained-corpus tier; native assertions remain in the E2E runner.
#[test]
#[ignore = "explicit full metadata corpus qualification"]
fn full_corpus_exact_emission() {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../sifr/tests/e2e/pass");
    sifr_corpus_oracle::full_corpus_exact_emission(&fixtures);
}
