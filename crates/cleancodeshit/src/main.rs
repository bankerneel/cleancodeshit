//! Preview CLI for Clean Code Shit 2.0.
//!
//! This milestone exposes metadata only. There is no scanner, planner,
//! cleanup command, process invocation or filesystem mutation here; those
//! arrive with the core API in later milestones.

#![forbid(unsafe_code)]

const HELP: &str = "\
cleancodeshit 2.0 preview

Usage: cleancodeshit [OPTION]

Options:
  -h, --help        Show this help and exit
  -V, --version     Show the version and exit
      --catalog     Show compiled-in catalog summary and exit

This preview build reports metadata only. It does not scan, plan or delete.
";

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(option) = args.next() else {
        print!("{HELP}");
        return;
    };

    match option.as_str() {
        "-h" | "--help" => print!("{HELP}"),
        "-V" | "--version" => println!("cleancodeshit {}", env!("CARGO_PKG_VERSION")),
        "--catalog" => catalog_summary(),
        other => {
            eprintln!("cleancodeshit: unrecognized option `{other}`\n");
            print!("{HELP}");
            std::process::exit(2);
        }
    }
}

fn catalog_summary() {
    match ccs_core::Catalog::builtin() {
        Ok(catalog) => {
            println!("schema version: {}", catalog.schema_version);
            println!("items: {}", catalog.items.len());
            println!("windows entries: {}", catalog.windows.len());
        }
        Err(error) => {
            eprintln!("cleancodeshit: invalid compiled-in catalog: {error}");
            std::process::exit(1);
        }
    }
}
