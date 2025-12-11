//! CLI definitions and command structures

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "slides")]
#[command(about = "Static site generator for HTML presentations")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new slides project in the current directory
    Init,

    /// Generate the HTML presentation
    Build {
        /// Deck name from decks.yaml
        #[arg(short, long, default_value = "default")]
        deck: String,

        /// Language for translations (e.g. "de", "en")
        #[arg(short, long, default_value = "en")]
        lang: String,
    },

    /// Watch for changes and rebuild automatically
    Watch {
        /// Deck name from decks.yaml
        #[arg(short, long, default_value = "default")]
        deck: String,

        /// Language for translations (e.g. "de", "en")
        #[arg(short, long, default_value = "en")]
        lang: String,
    },

    /// Create a new slides project
    New {
        /// Project name
        name: String,

        /// Path where the project should be created
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Extract images from PowerPoint presentations (.pptx)
    ImportImages {
        /// Path to PPTX file
        pptx: String,
    },

    /// Export the presentation as PDF
    Export {
        /// Deck name from decks.yaml
        #[arg(short, long, default_value = "default")]
        deck: String,

        /// Language for translations (e.g. "de", "en")
        #[arg(short, long, default_value = "en")]
        lang: String,

        /// Password for PDF protection. Without value, a secure password is generated.
        #[arg(short, long, num_args(0..=1), default_missing_value = "auto")]
        password: Option<String>,

        /// Disable printing
        #[arg(long)]
        no_print: bool,

        /// Disable text copying
        #[arg(long)]
        no_copy: bool,

        /// Output file
        #[arg(short, long, default_value = "output/presentation.pdf")]
        output: String,
    },
}
