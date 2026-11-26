mod bitbucket;
mod cli;
mod config;
mod rules;
mod types;

use anyhow::Result;
use clap::Parser;

use bitbucket::BitbucketClient;
use cli::Cli;
use config::read_config;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    if args.verbose {
        println!("Reading config from ~/.config/.unsub-nojira");
    }

    let config = read_config()?;

    if args.verbose {
        println!("Connecting to Bitbucket: {}", config.base_url);
    }

    let client = BitbucketClient::new(config.base_url, config.token);

    // Get list of repos to process
    let repos = match &args.repo_slug {
        Some(slug) => vec![slug.clone()],
        None => {
            if args.verbose {
                println!("Fetching all repositories in project {}", args.project_key);
            }
            let all_repos = client.list_repos(&args.project_key).await?;
            if args.verbose {
                println!("Found {} repositories", all_repos.len());
            }
            all_repos.into_iter().map(|r| r.slug).collect()
        }
    };

    let mut total_removed = 0;
    let mut total_skipped = 0;
    let mut total_prs_checked = 0;

    for repo_slug in &repos {
        // Skip excluded repositories
        if config.excluded_repos.contains(repo_slug) {
            if args.verbose {
                println!("\nSkipping excluded repository: {}", repo_slug);
            }
            continue;
        }

        if args.verbose {
            println!("\n=== Processing repository: {} ===", repo_slug);
        } else if repos.len() > 1 {
            println!("\nRepository: {}", repo_slug);
        }

        let prs = match client.list_open_prs(&args.project_key, repo_slug).await {
            Ok(prs) => prs,
            Err(e) => {
                eprintln!("Failed to fetch PRs from {}: {}", repo_slug, e);
                continue;
            }
        };

        if args.verbose {
            println!("Found {} open PRs", prs.len());
        }

        let nojira_prs: Vec<_> = prs
            .into_iter()
            .filter(|pr| pr.title.to_lowercase().contains("nojira"))
            .collect();

        if args.verbose {
            println!("Found {} PRs with 'NOJIRA' in title", nojira_prs.len());
        }

        if nojira_prs.is_empty() {
            if args.verbose {
                println!("No PRs found with 'NOJIRA' in the title.");
            }
            continue;
        }

        total_prs_checked += nojira_prs.len();

        for pr in &nojira_prs {
            if args.verbose {
                println!("\nChecking PR #{}: {}", pr.id, pr.title);
            }

            let participants = match client
                .get_pr_participants(&args.project_key, repo_slug, pr.id)
                .await
            {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to get participants for PR #{}: {}", pr.id, e);
                    continue;
                }
            };

            let is_participant = participants.iter().any(|p| p.user.slug == args.username);

            if !is_participant {
                if args.verbose {
                    println!("  Skipping: {} is not a participant", args.username);
                }
                total_skipped += 1;
                continue;
            }

            if args.dry_run {
                println!(
                    "[DRY RUN] Would remove {} from PR #{}: {}",
                    args.username, pr.id, pr.title
                );
                total_removed += 1;
            } else {
                match client
                    .remove_reviewer(&args.project_key, repo_slug, pr.id, &args.username)
                    .await
                {
                    Ok(_) => {
                        println!("Removed {} from PR #{}: {}", args.username, pr.id, pr.title);
                        total_removed += 1;
                    }
                    Err(e) => {
                        eprintln!("Failed to remove from PR #{}: {}", pr.id, e);
                    }
                }
            }
        }
    }

    println!("\n=== Summary ===");
    if repos.len() > 1 {
        println!("  Repositories processed: {}", repos.len());
    }
    println!("  Total PRs checked: {}", total_prs_checked);
    println!("  Removed from: {}", total_removed);
    println!("  Skipped: {}", total_skipped);

    Ok(())
}
