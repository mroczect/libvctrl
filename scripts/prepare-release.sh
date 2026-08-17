#!/usr/bin/env bash
set -euo pipefail

readonly RELEASE_JSON="release.json"
readonly LOG_FILE="release_$(date +%Y%m%d_%H%M%S).log"

# Warna
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[0;33m'
readonly NC='\033[0m'

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

check_prerequisites() {
    for cmd in cargo jq git gh; do
        if ! command -v "$cmd" &>/dev/null; then
            error_exit "$cmd not found. Please install it and ensure it is in PATH."
        fi
    done
}

ensure_clean_workspace() {
    if ! git diff-index --quiet HEAD --; then
        error_exit "Uncommitted changes found. Commit or stash them first."
    fi
    local branch
    branch=$(git rev-parse --abbrev-ref HEAD)
    if [ "$branch" != "master" ] && [ "$branch" != "main" ]; then
        log WARN "You are on branch '$branch', not master/main."
        read -r -p "Continue? (y/n) " confirm
        [ "$confirm" = "y" ] || exit 0
    fi
    git fetch origin
    local local_commit remote_commit
    local_commit=$(git rev-parse HEAD)
    remote_commit=$(git rev-parse "origin/$branch")
    if [ "$local_commit" != "$remote_commit" ]; then
        log WARN "Local branch is not up to date with origin/$branch."
        read -r -p "Continue anyway? (y/n) " confirm
        [ "$confirm" = "y" ] || exit 0
    fi
}

# Urutan publish yang benar (hardcoded, sesuai dependensi)
# libvctrl_sha512 -> libvctrl_handler -> libvctrl_core -> libvctrl -> libvctrl_plumbing -> libvctrl_porcelain
readonly CRATE_ORDER=(
    "libvctrl_sha512"
    "libvctrl_handler"
    "libvctrl_core"
    "libvctrl"
    "libvctrl_plumbing"
    "libvctrl_porcelain"
)

get_version() {
    local crate="$1"
    if [ ! -d "$crate" ]; then
        error_exit "Directory '$crate' does not exist"
    fi
    (cd "$crate" && cargo pkgid | cut -d'#' -f2 | cut -d: -f1)
}

prepare_crate() {
    local crate_name="$1"
    local version
    version=$(get_version "$crate_name")

    log INFO "-----------------------------------------------"
    log INFO "Preparing $crate_name v$version"

    # Build & test
    log INFO "Running tests for $crate_name..."
    if ! cargo test -p "$crate_name" --all-targets; then
        error_exit "Tests failed for $crate_name"
    fi

    # Clippy
    log INFO "Running clippy for $crate_name..."
    if ! cargo clippy -p "$crate_name" --all-targets -- -D warnings; then
        error_exit "Clippy failed for $crate_name"
    fi

    # Buat tag jika belum ada
    local tag="${crate_name}@${version}"
    if git rev-parse "$tag" >/dev/null 2>&1; then
        log WARN "Tag $tag already exists, skipping creation"
    else
        log INFO "Creating tag $tag"
        git tag -a "$tag" -m "Release $crate_name v$version"
    fi

    # Push tag
    log INFO "Pushing tag $tag"
    git push origin "$tag"
}

main() {
    log INFO "Release preparation started"
    check_prerequisites
    ensure_clean_workspace

    # Prepare all crates (test & tag)
    for crate in "${CRATE_ORDER[@]}"; do
        prepare_crate "$crate"
    done

    # Generate release.json based on Cargo.toml versions
    log INFO "Generating $RELEASE_JSON"
    {
        echo '{'
        echo '  "crates": ['
        first=true
        for crate in "${CRATE_ORDER[@]}"; do
            version=$(get_version "$crate")
            if [ "$first" = true ]; then
                first=false
                printf '    { "name": "%s", "version": "%s" }' "$crate" "$version"
            else
                printf ',\n    { "name": "%s", "version": "%s" }' "$crate" "$version"
            fi
        done
        echo ''
        echo '  ]'
        echo '}'
    } > "$RELEASE_JSON"

    # Commit release.json
    log INFO "Committing $RELEASE_JSON"
    git add "$RELEASE_JSON"
    git commit -m "release: add $RELEASE_JSON for publishing"

    # Push release.json
    log INFO "Pushing $RELEASE_JSON to origin"
    git push origin HEAD

    log INFO "All tags pushed and $RELEASE_JSON published. CI will now publish crates in order."
}

main "$@"
