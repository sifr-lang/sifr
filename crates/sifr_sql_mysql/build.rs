fn main() {
    println!("cargo:rerun-if-changed=src/mysql.lalrpop");
    if let Err(error) = lalrpop::Configuration::new()
        .use_cargo_dir_conventions()
        .emit_rerun_directives(false)
        .process()
    {
        panic!("cannot generate the provider-owned MySQL parser: {error}");
    }
}
