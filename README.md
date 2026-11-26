# unsub-nojira

A CLI tool to automatically remove yourself as a reviewer from Bitbucket Server PRs that don't need you:

- PRs with "NOJIRA" in the title
- PRs that already have at least 2 approvals from other people

## Features

- Remove yourself from specific repository or all repositories in a project
- Remove yourself from PRs with "NOJIRA" in the title (case-insensitive)
- Remove yourself from any open PR that already has 2+ approvals from others
- Dry-run mode to preview changes before applying
- Repository exclusion list via config file
- Pagination support for large repositories
- Verbose output for debugging

## Installation

### Build from source

```bash
cargo build --release
```

The binary will be available at `target/release/unsub-nojira`.

### Install globally

```bash
cargo install --path .
```

## Configuration

Create a configuration file at `~/.config/.unsub-nojira` with the following format:

```
https://bitbucket.example.com
your-access-token-here
excluded-repo-1
excluded-repo-2
```

**Line 1**: Bitbucket Server base URL

**Line 2**: Personal access token

**Line 3+**: Repository slugs to exclude (optional, one per line)

### Getting a Personal Access Token

1. Log into your Bitbucket Server instance
2. Go to **Manage Account** → **Personal access tokens**
3. Click **Create a token**
4. Give it a name and select **Repository read** and **Pull request write** permissions
5. Copy the generated token to line 2 of your config file

## Usage

### Basic Usage

> **Upgrading?** Run with `--dry-run` first. The approvals rule acts on all open PRs, not just NOJIRA ones.

Remove yourself from matching PRs in a specific repository:

```bash
unsub-nojira <username> <project-key> <repo-slug>
```

Remove yourself from matching PRs in ALL repositories in a project:

```bash
unsub-nojira <username> <project-key>
```

### Examples

```bash
# Single repository
unsub-nojira john.doe MYPROJ frontend

# All repositories in project
unsub-nojira john.doe MYPROJ

# Dry run (see what would be removed without making changes)
unsub-nojira john.doe MYPROJ --dry-run

# Verbose output
unsub-nojira john.doe MYPROJ --verbose

# Combine flags
unsub-nojira john.doe MYPROJ --dry-run --verbose
```

### Options

- `--dry-run`, `-d`: Show what would be removed without actually removing
- `--verbose`, `-v`: Show detailed output including skipped PRs
- `--help`, `-h`: Print help information

## How It Works

1. Reads configuration from `~/.config/.unsub-nojira`
2. Fetches list of repositories (if no repo specified)
3. For each repository:
   - Fetches all open pull requests
   - For each PR, removes you (unless in dry-run mode) if either rule matches:
     - **NOJIRA**: the title contains "NOJIRA" and you're a participant
     - **Approved**: at least 2 people *other than you* have approved, and you're on the reviewer list
   - The approvals rule never counts your own approval, so removing you never drops a PR below 2 approvals
   - The approvals rule leaves you on PRs where you've marked **Needs work**
   - If a PR matches both rules, it's reported as NOJIRA
4. Displays summary of operations

## Repository Exclusion

To exclude certain repositories from processing when scanning an entire project, add their slugs to the config file (lines 3+):

```
https://bitbucket.example.com
your-token
archived-project
legacy-codebase
test-sandbox
```

These repositories will be skipped when running without a specific repo argument.

## Output Examples

### Normal Mode

```
Repository: frontend
Removed john.doe from PR #123 [NOJIRA]: NOJIRA - Fix styling issue
Removed john.doe from PR #456 [2 approvals]: Add login page

Repository: backend
Removed john.doe from PR #789 [3 approvals]: Refactor billing

=== Summary ===
  Repositories processed: 2
  Total PRs checked: 12
  Removed (NOJIRA): 1
  Removed (approved): 2
  Left alone: 9
```

### Dry-Run Mode

```
[DRY RUN] Would remove john.doe from PR #123 [NOJIRA]: NOJIRA - Fix styling issue
[DRY RUN] Would remove john.doe from PR #456 [2 approvals]: Add login page

=== Summary ===
  Total PRs checked: 5
  Removed (NOJIRA): 1
  Removed (approved): 1
  Left alone: 3
```

### Verbose Mode

```
Reading config from ~/.config/.unsub-nojira
Connecting to Bitbucket: https://bitbucket.example.com
Fetching all repositories in project MYPROJ
Found 15 repositories

Skipping excluded repository: archived-project

=== Processing repository: frontend ===
Found 4 open PRs

Checking PR #123: NOJIRA - Fix styling issue
Removed john.doe from PR #123 [NOJIRA]: NOJIRA - Fix styling issue

Checking PR #456: Add login page
Removed john.doe from PR #456 [2 approvals]: Add login page

Checking PR #457: Fix flaky test
  Skipping: only 1 other approval(s), need 2

Checking PR #458: Bump version
  Skipping: john.doe is not a reviewer

=== Summary ===
  Repositories processed: 15
  Total PRs checked: 4
  Removed (NOJIRA): 1
  Removed (approved): 1
  Left alone: 2
```

## Error Handling

The tool provides clear error messages for common issues:

- **Config file not found**: Instructions on how to create it
- **Invalid credentials**: Authentication failure message
- **API errors**: Includes PR ID and error details
- **Repository access errors**: Continues with next repository

## Requirements

- Rust 2024 edition or later
- Access to Bitbucket Server (not Bitbucket Cloud)
- Valid personal access token with appropriate permissions
