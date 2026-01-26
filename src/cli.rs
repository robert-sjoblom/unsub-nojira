use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "unsub-nojira")]
#[command(about = "Remove yourself from Bitbucket PRs with 'NOJIRA' in the title", long_about = None)]
pub struct Cli {
    /// Bitbucket user slug (username)
    pub username: String,

    /// Bitbucket project key
    pub project_key: String,

    /// Repository slug (optional - if not provided, processes all repos in project)
    pub repo_slug: Option<String>,

    /// Show what would be removed without actually removing
    #[arg(short = 'd', long)]
    pub dry_run: bool,

    /// Show detailed output
    #[arg(short = 'v', long)]
    pub verbose: bool,
}
