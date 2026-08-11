#!/usr/bin/env bash
set -euo pipefail

###############################################################################
# Skrip Publikasi Otomatis untuk Workspace Rust
#
# Fitur:
#   - Memeriksa seluruh crate di workspace secara berurutan.
#   - Membandingkan versi lokal dengan crates.io.
#   - Menjalankan test dan clippy sebelum publikasi.
#   - Verifikasi bahwa semua dependensi lokal sudah terpublikasi.
#   - Opsi --dry-run untuk simulasi.
#   - Deteksi perubahan belum di-commit / branch tidak up-to-date.
#   - Timeout curl, retry dengan kode status HTTP.
#   - Membuat git tag dan GitHub release (dengan gh CLI).
#
# Persyaratan:
#   - cargo, jq, curl, git, gh (GitHub CLI) tersedia.
#   - gh auth login sudah dilakukan.
#   - cargo login sudah dilakukan (atau CARGO_REGISTRY_TOKEN disetel).
#   - Dijalankan dari root workspace.
###############################################################################

# Konfigurasi
readonly CURL_TIMEOUT=10
readonly CURL_RETRY=3
readonly LOG_FILE="publish_$(date +%Y%m%d_%H%M%S).log"

# Warna untuk output (non-emoji)
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[0;33m'
readonly NC='\033[0m' # No Color

DRY_RUN=0

# -----------------------------------------------------------------------------
# Fungsi logging
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
# Verifikasi alat bantu
# -----------------------------------------------------------------------------
check_prerequisites() {
    for cmd in cargo jq curl git gh; do
        if ! command -v "$cmd" &>/dev/null; then
            error_exit "$cmd tidak ditemukan. Pastikan sudah terinstal dan ada di PATH."
        fi
    done
    if ! gh auth status &>/dev/null; then
        error_exit "gh CLI belum terautentikasi. Jalankan 'gh auth login' terlebih dahulu."
    fi
}

# -----------------------------------------------------------------------------
# Cek repositori bersih
# -----------------------------------------------------------------------------
ensure_clean_workspace() {
    if ! git diff-index --quiet HEAD --; then
        error_exit "Ada perubahan yang belum di-commit. Commit atau stash terlebih dahulu."
    fi
    local branch
    branch=$(git rev-parse --abbrev-ref HEAD)
    if [ "$branch" != "master" ] && [ "$branch" != "main" ]; then
        log WARN "Anda berada di branch '$branch', bukan master/main."
        read -r -p "Lanjutkan? (y/n) " confirm
        if [ "$confirm" != "y" ]; then
            log INFO "Dibatalkan oleh pengguna."
            exit 0
        fi
    fi
    git fetch origin
    local local_commit remote_commit
    local_commit=$(git rev-parse HEAD)
    remote_commit=$(git rev-parse "origin/$branch")
    if [ "$local_commit" != "$remote_commit" ]; then
        log WARN "Branch lokal tidak sama dengan origin/$branch. Lakukan pull terlebih dahulu."
        read -r -p "Lanjutkan? (y/n) " confirm
        if [ "$confirm" != "y" ]; then
            log INFO "Dibatalkan oleh pengguna."
            exit 0
        fi
    fi
}

# -----------------------------------------------------------------------------
# Ambil daftar crate workspace dengan metadata (menghindari pipeline subshell)
# -----------------------------------------------------------------------------
get_workspace_members_array() {
    # Menggunakan process substitution agar loop berjalan di shell utama
    # (perlu lastpipe diaktifkan untuk bash, atau kita gunakan array)
    local -a members
    while IFS= read -r line; do
        members+=("$line")
    done < <(cargo metadata --no-deps --format-version 1 2>/dev/null \
        | jq -r '.packages[] | select(.publish != null) | "\(.name) \(.manifest_path)"')
    printf '%s\n' "${members[@]}"
}

# -----------------------------------------------------------------------------
# Ambil versi crate dari metadata JSON (lebih andal)
# -----------------------------------------------------------------------------
get_crate_version_from_metadata() {
    local crate_name="$1"
    cargo metadata --format-version 1 2>/dev/null \
        | jq -r --arg name "$crate_name" '.packages[] | select(.name == $name) | .version' \
        | head -1
}

# -----------------------------------------------------------------------------
# Cek apakah versi sudah ada di crates.io (dengan retry dan pengecekan HTTP)
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
            # Cek versi
            if jq -e --arg v "$version" '.versions[]? | select(.num == $v)' "$response_file" > /dev/null 2>&1; then
                rm -f "$response_file"
                return 0
            else
                rm -f "$response_file"
                return 1
            fi
        else
            log WARN "HTTP $http_code saat menghubungi crates.io untuk $crate_name (percobaan $attempt/$CURL_RETRY)"
            attempt=$((attempt + 1))
            sleep 1
        fi
    done

    rm -f "$response_file"
    log ERROR "Gagal menghubungi crates.io untuk $crate_name setelah $CURL_RETRY kali."
    return 1
}

# -----------------------------------------------------------------------------
# Cek dependensi: semua dependensi lokal harus sudah terpublikasi
# -----------------------------------------------------------------------------
check_local_deps_published() {
    local crate_name="$1"
    local deps
    # Ambil dependensi dengan path lokal
    deps=$(cargo metadata --format-version 1 2>/dev/null \
        | jq -r --arg name "$crate_name" '
            .packages[] | select(.name == $name) |
            .dependencies[]? | select(.path != null) | "\(.name) \(.req)"')

    while read -r dep_name dep_req; do
        [ -z "$dep_name" ] && continue
        # Dapatkan versi dependensi dari metadata workspace
        local dep_version
        dep_version=$(get_crate_version_from_metadata "$dep_name")
        if [ -z "$dep_version" ]; then
            log WARN "Dependensi $dep_name tidak ditemukan di workspace (mungkin bukan crate lokal)."
            continue
        fi
        if is_published "$dep_name" "$dep_version"; then
            log INFO "Dependensi $dep_name $dep_version sudah terpublikasi."
        else
            error_exit "Dependensi $dep_name $dep_version (dari $crate_name) belum dipublikasi. Publikasikan $dep_name terlebih dahulu."
        fi
    done <<< "$deps"
}

# -----------------------------------------------------------------------------
# Publikasi satu crate
# -----------------------------------------------------------------------------
publish_crate() {
    local crate_name="$1"
    local manifest_path="$2"
    local manifest_dir
    manifest_dir=$(dirname "$manifest_path")

    log INFO "==============================================="
    log INFO "Memproses: $crate_name"
    cd "$manifest_dir" || error_exit "Gagal masuk ke direktori $manifest_dir"

    local version
    version=$(get_crate_version_from_metadata "$crate_name")
    if [ -z "$version" ]; then
        cd - > /dev/null
        error_exit "Gagal membaca versi untuk $crate_name"
    fi
    log INFO "Versi: $version"

    if is_published "$crate_name" "$version"; then
        log INFO "$crate_name v$version sudah ada di crates.io, lewati."
        cd - > /dev/null
        return 0
    fi

    log INFO "$crate_name v$version belum dipublikasi. Melanjutkan..."

    # Cek dependensi lokal
    log INFO "Memeriksa dependensi lokal..."
    check_local_deps_published "$crate_name"

    # Jika dry-run, stop di sini
    if [ "$DRY_RUN" -eq 1 ]; then
        log INFO "[DRY-RUN] Akan menjalankan test, clippy, dan publish untuk $crate_name v$version"
        cd - > /dev/null
        return 0
    fi

    # Test & Clippy
    log INFO "Menjalankan test untuk $crate_name..."
    if ! cargo test -p "$crate_name" --all-targets; then
        cd - > /dev/null
        error_exit "Test gagal untuk $crate_name."
    fi
    log INFO "Menjalankan clippy untuk $crate_name..."
    if ! cargo clippy -p "$crate_name" --all-targets -- -D warnings; then
        cd - > /dev/null
        error_exit "Clippy gagal untuk $crate_name."
    fi
    log INFO "Test dan clippy berhasil."

    # Publish (non-interaktif)
    log INFO "Mempublikasi $crate_name v$version ke crates.io..."
    # Gunakan CARGO_REGISTRY_TOKEN jika ada, dan pipe "y" untuk konfirmasi
    local pub_output
    if pub_output=$(echo "y" | cargo publish -p "$crate_name" 2>&1); then
        log INFO "Publikasi berhasil."
    else
        log ERROR "Gagal mempublikasi $crate_name. Output: $pub_output"
        cd - > /dev/null
        error_exit "Gagal mempublikasi $crate_name."
    fi

    # Git tag
    local tag="${crate_name}@${version}"
    log INFO "Membuat git tag: $tag"
    if git rev-parse "$tag" >/dev/null 2>&1; then
        log WARN "Tag $tag sudah ada, melewati pembuatan tag."
    else
        git tag "$tag"
        git push origin "$tag"
        log INFO "Tag $tag dibuat dan didorong ke remote."
    fi

    # GitHub release
    log INFO "Membuat GitHub release untuk $tag..."
    if gh release view "$tag" &>/dev/null; then
        log WARN "Release $tag sudah ada, melewati pembuatan release."
    else
        gh release create "$tag" --generate-notes --title "$crate_name v$version"
        log INFO "GitHub release $tag dibuat."
    fi

    cd - > /dev/null
    log INFO "$crate_name selesai dipublikasi dan dirilis."
}

# -----------------------------------------------------------------------------
# Penggunaan
# -----------------------------------------------------------------------------
usage() {
    cat <<EOF
Penggunaan: $0 [OPTIONS] [CRATE_NAME]

Opsional:
  CRATE_NAME           Hanya proses crate tertentu.
  --dry-run            Simulasi, tidak melakukan perubahan apapun.

EOF
    exit 0
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    # Parse argumen
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

    log INFO "Memulai skrip publikasi otomatis..."
    if [ "$DRY_RUN" -eq 1 ]; then
        log WARN "MODE DRY-RUN AKTIF - Tidak ada perubahan yang dilakukan."
    fi

    check_prerequisites
    ensure_clean_workspace

    log INFO "Mengambil daftar crate workspace..."
    local members
    members=$(get_workspace_members_array)
    if [ -z "$members" ]; then
        error_exit "Tidak ada crate yang ditemukan di workspace."
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
            error_exit "Crate '$TARGET_CRATE' tidak ditemukan di workspace."
        fi
    else
        while IFS= read -r line; do
            crate_name=$(echo "$line" | awk '{print $1}')
            manifest_path=$(echo "$line" | awk '{print $2}')
            publish_crate "$crate_name" "$manifest_path"
        done <<< "$members"
    fi

    log INFO "Semua crate yang perlu dipublikasi telah diproses."
    if [ "$DRY_RUN" -eq 1 ]; then
        log WARN "MODE DRY-RUN: Tidak ada yang benar-benar dipublikasi."
    fi
}

main "$@"
