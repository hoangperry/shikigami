# shikigami-hook.ps1 — Windows entrypoint for the Claude Code hook bridge.
#
# Claude Code on Windows invokes hook commands via cmd.exe / PowerShell;
# pointing it directly at `python3 …\shikigami-hook.py` is fragile because
# Windows ships `python` (not `python3`) and the path quoting differs.
# This wrapper resolves the right interpreter once and forwards stdin
# to the cross-platform Python implementation, keeping a single source
# of truth for the actual transform logic.
#
# Usage (registered automatically by `scripts/install-hook.py`):
#   echo $env:CLAUDE_CODE_HOOK_JSON | powershell -ExecutionPolicy Bypass `
#       -File "$PSScriptRoot\shikigami-hook.ps1"

$ErrorActionPreference = 'SilentlyContinue'

# Resolve interpreter: prefer `python` (Windows convention), fall back to
# `python3` for users who installed via the python.org installer with
# the launcher disabled.
$python = $null
foreach ($candidate in @('python', 'python3', 'py')) {
    if (Get-Command $candidate -ErrorAction SilentlyContinue) {
        $python = $candidate
        break
    }
}
if (-not $python) {
    # Fail silently — never block Claude Code. Set SHIKIGAMI_DEBUG=1 to
    # surface the underlying issue.
    if ($env:SHIKIGAMI_DEBUG -eq '1') {
        Write-Error '[shikigami-hook.ps1] no python interpreter on PATH'
    }
    exit 0
}

$script = Join-Path $PSScriptRoot 'shikigami-hook.py'
if (-not (Test-Path $script)) {
    if ($env:SHIKIGAMI_DEBUG -eq '1') {
        Write-Error "[shikigami-hook.ps1] missing $script"
    }
    exit 0
}

# Forward stdin verbatim — Python script reads JSON from stdin, posts
# the transformed event, and swallows all errors.
$stdin = [Console]::In.ReadToEnd()
$stdin | & $python $script
exit 0
