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

# Urutan publish yang benar
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

generate_release_json() {
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
}

push_tags() {
    for crate in "${CRATE_ORDER[@]}"; do
        local version
        version=$(get_version "$crate")
        local tag="${crate}@${version}"

        # Cek remote dulu
        if git ls-remote --tags origin "refs/tags/${tag}" | grep -q "refs/tags/${tag}"; then
            log WARN "Tag $tag already exists on remote, skipping push"
            continue
        fi

        # Buat tag lokal jika belum ada
        if ! git rev-parse "$tag" >/dev/null 2>&1; then
            log INFO "Creating tag $tag"
            git tag -a "$tag" -m "Release $crate v$version"
        fi

        log INFO "Pushing tag $tag"
        git push origin "$tag"
    done
}

main() {
    log INFO "Release preparation started"
    check_prerequisites
    ensure_clean_workspace

    # 1. Generate release.json
    generate_release_json

    # 2. Create new branch
    local branch_name
    branch_name="release/$(date +%Y%m%d%H%M%S)"
    log INFO "Creating branch $branch_name"
    git checkout -b "$branch_name"

    # 3. Commit release.json
    log INFO "Committing $RELEASE_JSON (force add)"
    git add -f "$RELEASE_JSON"
    git commit -m "chore(release): add $RELEASE_JSON for ordered publishing"

    # 4. Push branch
    log INFO "Pushing branch $branch_name"
    git push -u origin "$branch_name"

    # 5. Create PR
    log INFO "Creating pull request"
    cat > /tmp/pr_body.md <<EOF
## Summary
Add release.json untuk publish crate berurutan ke crates.io.

## Crates
$(for crate in "${CRATE_ORDER[@]}"; do
    echo "- $crate v$(get_version "$crate")"
done)

## Note
- Setelah PR ini di-merge, workflow publish akan membaca release.json dan mempublish crate sesuai urutan.
- Pastikan tag untuk setiap crate sudah di-push sebelum merge (atau jalankan script ini dengan argumen --push-tags).
EOF

    local pr_url
    pr_url=$(gh pr create \
        --title "chore(release): add release.json for ordered publishing" \
        --body-file /tmp/pr_body.md \
        --base "$(git rev-parse --abbrev-ref origin/HEAD | sed 's|origin/||')" \
        --head "$branch_name")

    local pr_number
    pr_number=$(basename "$pr_url")

    # 6. Interactive confirmation for tags
    read -r -p "Do you want to push release tags now? (y/n) " confirm
    if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
        push_tags
    else
        log INFO "Tags not pushed. You can push them later with: bash scripts/prepare-release.sh --push-tags"
    fi

    log INFO "Pull request created: $pr_url"
    log INFO "All done. Merge PR #$pr_number to trigger publishing."
}

# Allow --push-tags mode
if [ "${1:-}" = "--push-tags" ]; then
    check_prerequisites
    push_tags
    exit 0
fi

main "$@"
