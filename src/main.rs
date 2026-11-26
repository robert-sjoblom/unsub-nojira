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
use rules::{Decision, MIN_OTHER_APPROVALS, decide};
use types::PullRequest;

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

    let mut total_removed_nojira = 0;
    let mut total_removed_approved = 0;
    let mut total_left_alone = 0;
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
                eprintln!("Failed to fetch PRs from {}: {:#}", repo_slug, e);
                continue;
            }
        };

        if args.verbose {
            println!("Found {} open PRs", prs.len());
        }

        total_prs_checked += prs.len();

        for pr in &prs {
            if args.verbose {
                println!("\nChecking PR #{}: {}", pr.id, pr.title);
            }

            match decide(pr, &args.username) {
                Decision::Nojira => {
                    let participants = match client
                        .get_pr_participants(&args.project_key, repo_slug, pr.id)
                        .await
                    {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to get participants for PR #{}: {:#}", pr.id, e);
                            continue;
                        }
                    };

                    let is_participant = participants.iter().any(|p| p.user.slug == args.username);

                    if !is_participant {
                        if args.verbose {
                            println!("  Skipping: {} is not a participant", args.username);
                        }
                        total_left_alone += 1;
                        continue;
                    }

                    if remove_from_pr(&client, &args, repo_slug, pr, "NOJIRA").await {
                        total_removed_nojira += 1;
                    }
                }
                Decision::Approved { others } => {
                    let tag = format!("{others} approvals");
                    if remove_from_pr(&client, &args, repo_slug, pr, &tag).await {
                        total_removed_approved += 1;
                    }
                }
                Decision::NotReviewer => {
                    if args.verbose {
                        println!("  Skipping: {} is not a reviewer", args.username);
                    }
                    total_left_alone += 1;
                }
                Decision::NeedsWork => {
                    if args.verbose {
                        println!("  Skipping: {} marked Needs work", args.username);
                    }
                    total_left_alone += 1;
                }
                Decision::TooFewApprovals { others } => {
                    if args.verbose {
                        println!(
                            "  Skipping: only {others} other approval(s), need {MIN_OTHER_APPROVALS}"
                        );
                    }
                    total_left_alone += 1;
                }
            }
        }
    }

    println!("\n=== Summary ===");
    if repos.len() > 1 {
        println!("  Repositories processed: {}", repos.len());
    }
    println!("  Total PRs checked: {}", total_prs_checked);
    println!("  Removed (NOJIRA): {}", total_removed_nojira);
    println!("  Removed (approved): {}", total_removed_approved);
    println!("  Left alone: {}", total_left_alone);

    Ok(())
}

/// Removes the user from the PR (or reports what would happen in dry-run).
/// Returns true if the user was removed, or would have been.
async fn remove_from_pr(
    client: &BitbucketClient,
    args: &Cli,
    repo_slug: &str,
    pr: &PullRequest,
    tag: &str,
) -> bool {
    if args.dry_run {
        println!(
            "[DRY RUN] Would remove {} from PR #{} [{}]: {}",
            args.username, pr.id, tag, pr.title
        );
        return true;
    }

    match client
        .remove_reviewer(&args.project_key, repo_slug, pr.id, &args.username)
        .await
    {
        Ok(()) => {
            println!(
                "Removed {} from PR #{} [{}]: {}",
                args.username, pr.id, tag, pr.title
            );
            true
        }
        Err(e) => {
            eprintln!("Failed to remove from PR #{}: {:#}", pr.id, e);
            false
        }
    }
}
