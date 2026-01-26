# unsub-nojira

A CLI tool to automatically remove yourself from Bitbucket Server PRs with "NOJIRA" in the title.

## Features

- Remove yourself from specific repository or all repositories in a project
- Filter PRs by "NOJIRA" in title (case-insensitive)
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

Remove yourself from NOJIRA PRs in a specific repository:

```bash
unsub-nojira <username> <project-key> <repo-slug>
```

Remove yourself from NOJIRA PRs in ALL repositories in a project:

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
   - Filters PRs with "NOJIRA" in the title
   - Checks if you're a participant on each PR
   - Removes you as a reviewer (unless in dry-run mode)
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
Removed john.doe from PR #123: NOJIRA - Fix styling issue
Removed john.doe from PR #456: NOJIRA - Update dependencies

Repository: backend
Removed john.doe from PR #789: NOJIRA - Quick hotfix

=== Summary ===
  Repositories processed: 2
  Total PRs checked: 3
  Removed from: 3
  Skipped: 0
```

### Dry-Run Mode

```
Repository: frontend
[DRY RUN] Would remove john.doe from PR #123: NOJIRA - Fix styling issue
[DRY RUN] Would remove john.doe from PR #456: NOJIRA - Update dependencies

=== Summary ===
  Repositories processed: 1
  Total PRs checked: 2
  Removed from: 2
  Skipped: 0
```

### Verbose Mode

```
Reading config from ~/.config/.unsub-nojira
Connecting to Bitbucket: https://bitbucket.example.com
Fetching all repositories in project MYPROJ
Found 15 repositories

Skipping excluded repository: archived-project

=== Processing repository: frontend ===
Found 50 open PRs
Found 2 PRs with 'NOJIRA' in title

Checking PR #123: NOJIRA - Fix styling issue
Removed john.doe from PR #123: NOJIRA - Fix styling issue

Checking PR #456: NOJIRA - Update dependencies
  Skipping: john.doe is not a participant

=== Summary ===
  Repositories processed: 1
  Total PRs checked: 2
  Removed from: 1
  Skipped: 1
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
