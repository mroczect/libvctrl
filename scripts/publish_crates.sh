#!/usr/bin/env bash
set -euo pipefail

###############################################################################
# Automated Crate Publication Script for a Rust Workspace
#
# Features:
#   - Checks all crates in the workspace sequentially.
#   - Compares the local version with crates.io.
#   - Runs tests and clippy before publishing.
#   - Verifies that all local dependencies are already published.
#   - --dry-run option for simulation.
#   - Detects uncommitted changes and branch sync status.
#   - Curl timeout, retry with HTTP status code checking.
#   - Creates a git tag and GitHub release (using gh CLI).
#
# Requirements:
#   - cargo, jq, curl, git, gh (GitHub CLI) are installed.
#   - gh auth login has been performed.
#   - cargo login has been performed (or CARGO_REGISTRY_TOKEN is set).
#   - Run from the workspace root.
###############################################################################

# Configuration
readonly CURL_TIMEOUT=10
readonly CURL_RETRY=3
readonly LOG_FILE="publish_$(date +%Y%m%d_%H%M%S).log"

# Output colours
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[0;33m'
readonly NC='\033[0m' # No Color

DRY_RUN=0

# -----------------------------------------------------------------------------
# Logging functions
# -----------------------------------------------------------------------------
log() {
    local level="$1"; shift
    local msg="$*"
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    case "$level" in
        INFO)  printf "[%s] ${GREEN}INFO${NC}  %s\n" "$timestamp" "$msg" | tee -a "$LOG_FILE" ;;
        WARN)  printf "[%s] ${YELLOW}WARN${NC}  %s\n" "$timestamp" "$msg" | tee -a "$LOG_FILE" ;;
        ERROR) printf "[%s] ${RED}ERROR${NC} %s\n" "$timestamp" "$msg" | tee -a "$LOG_FILE" >&2 ;;
        *)     printf "[%s] %s\n" "$timestamp" "$msg" | tee -a "$LOG_FILE" ;;
    esac
}

error_exit() {
    log ERROR "$1"
    exit 1
}

# -----------------------------------------------------------------------------
# Verify required tools
# -----------------------------------------------------------------------------
check_prerequisites() {
    for cmd in cargo jq curl git gh; do
        if ! command -v "$cmd" &>/dev/null; then
            error_exit "$cmd not found. Please install it and ensure it is in PATH."
        fi
    done
    if ! gh auth status &>/dev/null; then
        error_exit "gh CLI not authenticated. Run 'gh auth login' first."
    fi
}

# -----------------------------------------------------------------------------
# Ensure the repository is clean and up to date
# -----------------------------------------------------------------------------
ensure_clean_workspace() {
    if ! git diff-index --quiet HEAD --; then
        error_exit "There are uncommitted changes. Commit or stash them first."
    fi
    local branch
    branch=$(git rev-parse --abbrev-ref HEAD)
    if [ "$branch" != "master" ] && [ "$branch" != "main" ]; then
        log WARN "You are on branch '$branch', not master/main."
        read -r -p "Continue? (y/n) " confirm
        if [ "$confirm" != "y" ]; then
            log INFO "Aborted by user."
            exit 0
        fi
    fi
    git fetch origin
    local local_commit remote_commit
    local_commit=$(git rev-parse HEAD)
    remote_commit=$(git rev-parse "origin/$branch")
    if [ "$local_commit" != "$remote_commit" ]; then
        log WARN "Local branch is not up to date with origin/$branch. Pull first."
        read -r -p "Continue anyway? (y/n) " confirm
        if [ "$confirm" != "y" ]; then
            log INFO "Aborted by user."
            exit 0
        fi
    fi
}

# -----------------------------------------------------------------------------
# Retrieve workspace members without subshell variable loss
# -----------------------------------------------------------------------------
get_workspace_members_array() {
    local -a members
    while IFS= read -r line; do
        members+=("$line")
    done < <(cargo metadata --no-deps --format-version 1 2>/dev/null \
        | jq -r '.packages[] | select(.publish != null) | "\(.name) \(.manifest_path)"')
    printf '%s\n' "${members[@]}"
}

# -----------------------------------------------------------------------------
# Get crate version from cargo metadata (robust)
# -----------------------------------------------------------------------------
get_crate_version_from_metadata() {
    local crate_name="$1"
    cargo metadata --format-version 1 2>/dev/null \
        | jq -r --arg name "$crate_name" '.packages[] | select(.name == $name) | .version' \
        | head -1
}

# -----------------------------------------------------------------------------
# Check if a version is already published on crates.io (with HTTP retry)
# -----------------------------------------------------------------------------
is_published() {
    local crate_name="$1"
    local version="$2"
    local attempt=1
    local http_code
    local response_file
    response_file=$(mktemp)

    while [ $attempt -le $CURL_RETRY ]; do
        http_code=$(curl -sS --max-time "$CURL_TIMEOUT" -w "%{http_code}" -o "$response_file" \
            "https://crates.io/api/v1/crates/$crate_name" 2>/dev/null || true)
        if [ "$http_code" = "200" ]; then
            if jq -e --arg v "$version" '.versions[]? | select(.num == $v)' "$response_file" > /dev/null 2>&1; then
                rm -f "$response_file"
                return 0
            else
                rm -f "$response_file"
                return 1
            fi
        else
            log WARN "HTTP $http_code when contacting crates.io for $crate_name (attempt $attempt/$CURL_RETRY)"
            attempt=$((attempt + 1))
            sleep 1
        fi
    done

    rm -f "$response_file"
    log ERROR "Unable to reach crates.io for $crate_name after $CURL_RETRY attempts."
    return 1
}

# -----------------------------------------------------------------------------
# Ensure all local dependencies are published before publishing a crate
# -----------------------------------------------------------------------------
check_local_deps_published() {
    local crate_name="$1"
    local deps
    deps=$(cargo metadata --format-version 1 2>/dev/null \
        | jq -r --arg name "$crate_name" '
            .packages[] | select(.name == $name) |
            .dependencies[]? | select(.path != null) | "\(.name) \(.req)"')

    while read -r dep_name dep_req; do
        [ -z "$dep_name" ] && continue
        local dep_version
        dep_version=$(get_crate_version_from_metadata "$dep_name")
        if [ -z "$dep_version" ]; then
            log WARN "Dependency $dep_name not found in workspace (may not be a local crate)."
            continue
        fi
        if is_published "$dep_name" "$dep_version"; then
            log INFO "Dependency $dep_name $dep_version is already published."
        else
            error_exit "Dependency $dep_name $dep_version (required by $crate_name) is not published. Publish $dep_name first."
        fi
    done <<< "$deps"
}

# -----------------------------------------------------------------------------
# Publish a single crate
# -----------------------------------------------------------------------------
publish_crate() {
    local crate_name="$1"
    local manifest_path="$2"
    local manifest_dir
    manifest_dir=$(dirname "$manifest_path")

    log INFO "==============================================="
    log INFO "Processing: $crate_name"
    cd "$manifest_dir" || error_exit "Failed to enter directory $manifest_dir"

    local version
    version=$(get_crate_version_from_metadata "$crate_name")
    if [ -z "$version" ]; then
        cd - > /dev/null
        error_exit "Failed to read version for $crate_name"
    fi
    log INFO "Version: $version"

    if is_published "$crate_name" "$version"; then
        log INFO "$crate_name v$version is already on crates.io, skipping."
        cd - > /dev/null
        return 0
    fi

    log INFO "$crate_name v$version not yet published. Proceeding..."

    # Check local dependencies
    log INFO "Checking local dependencies..."
    check_local_deps_published "$crate_name"

    # Dry-run mode
    if [ "$DRY_RUN" -eq 1 ]; then
        log INFO "[DRY-RUN] Would run tests, clippy, and publish for $crate_name v$version"
        cd - > /dev/null
        return 0
    fi

    # Test & Clippy
    log INFO "Running tests for $crate_name..."
    if ! cargo test -p "$crate_name" --all-targets; then
        cd - > /dev/null
        error_exit "Tests failed for $crate_name."
    fi
    log INFO "Running clippy for $crate_name..."
    if ! cargo clippy -p "$crate_name" --all-targets -- -D warnings; then
        cd - > /dev/null
        error_exit "Clippy failed for $crate_name."
    fi
    log INFO "Tests and clippy passed."

    # Publish (non-interactive)
    log INFO "Publishing $crate_name v$version to crates.io..."
    local pub_output
    if pub_output=$(echo "y" | cargo publish -p "$crate_name" 2>&1); then
        log INFO "Publication successful."
    else
        log ERROR "Failed to publish $crate_name. Output: $pub_output"
        cd - > /dev/null
        error_exit "Publication failed for $crate_name."
    fi

    # Git tag
    local tag="${crate_name}@${version}"
    log INFO "Creating git tag: $tag"
    if git rev-parse "$tag" >/dev/null 2>&1; then
        log WARN "Tag $tag already exists, skipping tag creation."
    else
        git tag "$tag"
        git push origin "$tag"
        log INFO "Tag $tag created and pushed."
    fi

    # GitHub release
    log INFO "Creating GitHub release for $tag..."
    if gh release view "$tag" &>/dev/null; then
        log WARN "Release $tag already exists, skipping."
    else
        gh release create "$tag" --generate-notes --title "$crate_name v$version"
        log INFO "GitHub release $tag created."
    fi

    cd - > /dev/null
    log INFO "$crate_name successfully published and released."
}

# -----------------------------------------------------------------------------
# Usage
# -----------------------------------------------------------------------------
usage() {
    cat <<EOF
Usage: $0 [OPTIONS] [CRATE_NAME]

Options:
  CRATE_NAME           Process only the specified crate.
  --dry-run            Simulate, do not make any changes.
  -h, --help           Show this help message.
EOF
    exit 0
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    # Parse arguments
    TARGET_CRATE=""
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --dry-run)
                DRY_RUN=1
                shift
                ;;
            -h|--help)
                usage
                ;;
            *)
                TARGET_CRATE="$1"
                shift
                ;;
        esac
    done

    log INFO "Starting automated publication script..."
    if [ "$DRY_RUN" -eq 1 ]; then
        log WARN "DRY-RUN MODE ACTIVE - No changes will be made."
    fi

    check_prerequisites
    ensure_clean_workspace

    log INFO "Retrieving workspace crate list..."
    local members
    members=$(get_workspace_members_array)
    if [ -z "$members" ]; then
        error_exit "No crates found in the workspace."
    fi

    if [ -n "$TARGET_CRATE" ]; then
        local found=0
        while IFS= read -r line; do
            crate_name=$(echo "$line" | awk '{print $1}')
            manifest_path=$(echo "$line" | awk '{print $2}')
            if [ "$crate_name" = "$TARGET_CRATE" ]; then
                found=1
                publish_crate "$crate_name" "$manifest_path"
                break
            fi
        done <<< "$members"

        if [ "$found" -eq 0 ]; then
            error_exit "Crate '$TARGET_CRATE' not found in workspace."
        fi
    else
        while IFS= read -r line; do
            crate_name=$(echo "$line" | awk '{print $1}')
            manifest_path=$(echo "$line" | awk '{print $2}')
            publish_crate "$crate_name" "$manifest_path"
        done <<< "$members"
    fi

    log INFO "All necessary crates have been processed."
    if [ "$DRY_RUN" -eq 1 ]; then
        log WARN "DRY-RUN: Nothing was actually published."
    fi
}

main "$@"
