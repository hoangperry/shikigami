# Windows Code-Signing Runbook

> **Status**: scaffolding shipped, cert procurement deferred.
> **Tracks**: GitHub issue [#30](https://github.com/hoangperry/shikigami/issues/30).

This runbook documents how to take Shikigami's Windows build from the
current "unsigned, SmartScreen-warns" state to "signed, immediate-trust"
when an EV (Extended Validation) code-signing certificate becomes
available. The CI workflow (`.github/workflows/release.yml`) already
contains a gated signing path — flipping it on is a secrets-only
operation, no code changes required.

---

## Why we don't sign today

| Cert tier | Cost / yr | SmartScreen reputation | Recommendation |
|---|---|---|---|
| **Standard OV** (Sectigo / DigiCert) | ~$60–120 | Builds gradually over weeks of installs | Useful only if signed-but-warns is acceptable for several weeks per release |
| **EV (Extended Validation)** | ~$300–400 | **Immediate** trust | Required for instant SmartScreen pass |
| **Microsoft Store**-distributed | $19/yr individual | N/A — Store does signing | Different distribution channel; not yet on the roadmap |

For an alpha-stage repo with a small install base, neither tier pays
off versus the SmartScreen "More info → Run anyway" workaround
documented in the README. Revisit when v0.2 stabilises with non-trivial
install numbers, or when the project is sponsored by an org that
already has an EV setup.

---

## When to sign — checklist

Defer signing until **all** of these are true:

- [ ] v0.2.0 has shipped a stable (non-alpha) tag at least once
- [ ] Install base exceeds ~100 unique users (telemetry-free estimate
      from GitHub Release download counts)
- [ ] A maintainer or sponsoring org owns either an existing EV cert
      we can borrow signing cycles from, or has approved the
      ~$400/yr recurring cost
- [ ] An EV-cert-aware human is on call to respond to revocation
      events (cert leakage = full re-issue + republish)

If any item is unchecked, the unsigned + xattr-equivalent path is the
right answer.

---

## Procurement

### 1. Choose a CA

Reputable EV code-signing CAs (alphabetical):
- **DigiCert** — well-trusted, premium pricing
- **GlobalSign** — competitive EV pricing
- **Sectigo (formerly Comodo)** — cheapest tier; reputation is fine
- **SSL.com** — hardware-token-free option (HSM service)

Avoid resellers; they sometimes ship 1- or 2-year certs at the same
price as direct CA 1-year tier. Direct purchase is more flexible.

### 2. Identity verification

EV requires a **legal entity** (LLC / corporation / sole proprietor with
DBA registration). Personal certificates are not eligible at the EV tier
— only OV. The CA will request:

- Articles of incorporation / business license
- Phone-number verification at the registered address (Dun & Bradstreet
  or LexisNexis listing required; CA will create one if missing)
- Notarised identity check for the signer (the human who'll handle the
  hardware token)

Budget **5–10 business days** for first-time verification.

### 3. Hardware token vs HSM service

- **Hardware token (USB)** — cheapest. Cert lives on a FIPS 140-2 USB
  token shipped by the CA. Plug into the build machine to sign; can't
  be exported. Works for local dev signing but **awkward for CI** —
  you'd need a self-hosted runner with the token plugged in.
- **HSM service** (e.g. SSL.com eSigner, DigiCert KeyLocker) — cert is
  hosted on the CA's HSM; signing happens via API. Works on **GitHub
  Actions hosted runners** without physical hardware. Costs slightly
  more (~$100/yr extra) but is the only viable option for our current
  CI flow.

**Pick HSM service** unless someone is willing to operate a self-hosted
runner.

---

## CI integration (when ready)

The release workflow already has the import + sign steps; they're
gated by the `WINDOWS_CERTIFICATE` secret being non-empty. Concretely:

### For HSM-service certs

The CA provides a credential file (e.g. SSL.com's `eSigner` config) +
API credentials. Adapt the "Import Windows signing certificate" step to
call the HSM SDK instead of `Import-PfxCertificate`. Tauri's bundler
invokes `signtool` which can be pointed at HSM-backed keys via vendor
SDKs.

### For .pfx file (USB-token export, OV tier)

1. Export the cert + private key to a `.pfx` file (one-time, from the
   shipped USB token using `certmgr` or vendor tooling).
2. Base64-encode and store in `WINDOWS_CERTIFICATE` GitHub Actions secret:
   ```bash
   base64 -i shikigami-signing.pfx | pbcopy   # macOS
   ```
3. Store the .pfx password in `WINDOWS_CERTIFICATE_PASSWORD` secret.
4. Push a tag to trigger the release workflow — the import step will
   fire, the build step will sign automatically.

**Do not commit the .pfx or its base64 to source under any circumstance.**
If it leaks, revoke immediately via CA portal and re-issue.

---

## Verifying a signed build

After CI produces signed artifacts:

```powershell
# Inspect the signature on a downloaded .exe
Get-AuthenticodeSignature .\Shikigami_*_x64-setup.exe | Format-List

# Confirm SmartScreen treats it as trusted
# (browse to Release page, download, observe NO "Windows protected your PC" dialog)
```

For the `.msi`, signed status is visible in *Properties → Digital
Signatures*.

---

## Revocation runbook

If the cert key is suspected leaked:

1. **Immediately** notify the CA via emergency revocation channel
   (most CAs publish a 24/7 line for this — find it before you need it)
2. Remove `WINDOWS_CERTIFICATE` + `WINDOWS_CERTIFICATE_PASSWORD` GitHub
   secrets
3. Cut a new release with `bundle.windows.certificateThumbprint` reset
   to `null` so unsigned-build path resumes
4. Add a SECURITY.md note + GitHub Issue documenting the incident date
5. Coordinate re-issue with the CA (typically 24-72h)

---

## See also

- [Tauri Windows distribution guide](https://v2.tauri.app/distribute/sign/windows/)
- Issue [#30](https://github.com/hoangperry/shikigami/issues/30) — tracks the procurement decision
- [`.github/workflows/release.yml`](../.github/workflows/release.yml) — the CI scaffolding this doc activates
