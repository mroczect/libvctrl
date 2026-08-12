# Security Policy

## Supported Versions

The following table lists the crates in the `libvctrl` workspace and the
versions for which security patches are provided. Only the latest **stable**
release of each crate is actively supported. Older versions are not
guaranteed to receive security fixes.

| Crate                | Supported Versions |
| -------------------- | ------------------ |
| `libvctrl`           | 2.x (latest)       |
| `libvctrl_handler`   | 4.x (latest)       |
| `libvctrl_core`      | 2.x (latest)       |
| `libvctrl_sha512`    | 2.x (latest)       |
| `libvctrl_plumbing`  | 0.x (latest)       |
| `libvctrl_porcelain` | Not yet released   |

If you are using an older version, we strongly recommend upgrading to the
latest release.

## Security Design

_All crates in this workspace use `#![forbid(unsafe_code)]`._ This reduces
the attack surface for memory‑safety vulnerabilities. Additionally,
traversal‑prone operations (such as constructing `TreeEntry` names) are
guarded by validation functions that reject `"/"`, `"."`, and `".."`.

## Reporting a Vulnerability

We appreciate the community’s effort in disclosing security issues
responsibly. **Please do not report security vulnerabilities through public
GitHub issues.**

Instead, use the **private reporting** feature built into GitHub:

1. Go to the [Security tab](../../security) of this repository.
2. Click **Report a vulnerability**.
3. Fill in the form with:
   - A clear description of the vulnerability.
   - Steps to reproduce.
   - The affected crate and version(s).
   - Any potential impact or exploit scenarios.

Alternatively, you may send an encrypted email to the maintainer. The
maintainer’s PGP key is available in the root of this repository
(`pgp-key.asc`). Contact: `mroczect@proton.me` (replace with the real
address if different).

You can also use the GitHub Security Advisory "Request CVE" option if
appropriate.

## What to Expect

- **Acknowledgment** – We will acknowledge your report within **48 hours**.
- **Investigation** – We will investigate and attempt to reproduce the issue.
- **Fix Development** – If confirmed, we will develop a fix and release a
  patch for the supported version(s).
- **Credit** – With your permission, we will credit you in the release notes
  and security advisory.

We follow **coordinated disclosure**: once a fix is available, we will
publish a GitHub Security Advisory and encourage users to upgrade.

## Disclosure Policy

- **Public disclosure** occurs after a patch has been published and users
  have had reasonable time to upgrade (usually 7 days after the release).
- **Third‑party dependencies** – If the vulnerability originates from a
  third‑party crate, we will work with that crate’s maintainers and follow
  their disclosure timeline.

## Preferred Languages

We accept reports in **English** or **Indonesian** (Bahasa Indonesia).

---

_This policy is inspired by the [GitHub Security Policy
template](https://docs.github.com/en/code-security/getting-started/adding-a-security-policy-to-your-repository)._
