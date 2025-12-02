mod cli;
mod model;
mod project;
mod render;
mod translations;
mod util;

use clap::Parser;
use cli::{Cli, Commands};
use model::Deck;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Init) => project::init(),
        Some(Commands::Build { deck, lang }) => Deck::new(&deck, &lang).build(),
        Some(Commands::Watch { deck, lang }) => Deck::new(&deck, &lang).watch(),
        Some(Commands::New { name, path }) => project::create(&name, path.as_deref()),
        Some(Commands::Import { pptx, images_only }) => project::import(&pptx, images_only),
        Some(Commands::Export {
            deck,
            lang,
            password,
            no_print,
            no_copy,
            output,
        }) => Deck::new(&deck, &lang).export(password, no_print, no_copy, &output),

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
