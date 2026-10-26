use std::process::ExitCode;

fn main() -> ExitCode {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() == 1 && args[0] == "--version" {
        println!("sifr-corpus-oracle {}", sifr_sysroot::COMPILER_SIFR_VERSION);
        return ExitCode::SUCCESS;
    }
    if args.len() != 2 || args[0] != "--fixtures" {
        eprintln!("usage: sifr_corpus_oracle --fixtures <directory> | --version");
        return ExitCode::from(2);
    }
    sifr_corpus_oracle::full_corpus_exact_emission(std::path::Path::new(&args[1]));
    ExitCode::SUCCESS
}
