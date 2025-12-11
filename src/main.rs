mod cli;

use clap::Parser;
use cli::{Cli, Commands};
use env_logger::Env;
use slides_rs::model::Project;

fn main() {
    // Initialize logger with default level of "warn" for warnings to show
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Init) => Project::init().map(|_| ()),
        Some(Commands::New { name, path }) => Project::create(&name, path.as_deref()).map(|_| ()),

        Some(Commands::Build { deck, lang }) => {
            Project::current().and_then(|p| p.deck(&deck, &lang).build())
        }
        Some(Commands::Watch { deck, lang }) => {
            Project::current().and_then(|p| p.deck(&deck, &lang).watch())
        }
        Some(Commands::Export {
            deck,
            lang,
            password,
            no_print,
            no_copy,
            output,
        }) => Project::current().and_then(|p| {
            p.deck(&deck, &lang).build()?;
            p.export_pdf(password, no_print, no_copy, &output)
        }),
        Some(Commands::ImportImages { pptx }) => {
            Project::current().and_then(|p| p.import_images(&pptx))
        }

        None => {
            eprintln!("No subcommand provided. Use 'slides --help' for usage information.");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}
