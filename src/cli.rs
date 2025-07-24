use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "karabiner-pkl")]
#[command(author, version, about = "Karabiner configuration using Apple Pkl", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true, default_value = "~/.config/karabiner.pkl")]
    pub config: String,

    #[arg(short, long, global = true)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Start {
        #[arg(short, long)]
        foreground: bool,
    },

    Stop,

    Compile {
        #[arg(
            short,
            long,
            help = "Override the profile name (default: uses config value or 'pkl')"
        )]
        profile_name: Option<String>,
    },

    Check,

    Logs {
        #[arg(short, long, default_value = "50")]
        lines: usize,

        #[arg(short, long)]
        follow: bool,
    },

    Status,

    Init {
        #[arg(short, long)]
        force: bool,
    },

    Add {
        #[arg(help = "Path to a local .pkl file or URL to import")]
        source: String,

        #[arg(
            short,
            long,
            help = "Name for the imported file (defaults to source filename)"
        )]
        name: Option<String>,
    },
}
