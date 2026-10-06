# installed by herdr
# managed by herdr; reinstalling or updating the integration overwrites this file.
# add custom hooks beside this file instead of editing it.
# HERDR_INTEGRATION_ID=kiro
# HERDR_INTEGRATION_VERSION=1

param([string]$Action = "")

if ($Action -ne "session") { exit 0 }
if ($env:HERDR_ENV -ne "1") { exit 0 }
if ([string]::IsNullOrWhiteSpace($env:HERDR_PANE_ID)) { exit 0 }
if ([string]::IsNullOrWhiteSpace($env:HERDR_SOCKET_PATH)) { exit 0 }

$inputText = [Console]::In.ReadToEnd()
try {
    $payload = if ([string]::IsNullOrWhiteSpace($inputText)) { $null } else { $inputText | ConvertFrom-Json }
} catch {
    $payload = $null
}
if ($null -eq $payload) { exit 0 }
$eventName = if ($payload.hook_event_name) { $payload.hook_event_name } else { $payload.trigger }
if ($eventName -and $eventName -notin @("SessionStart", "agentSpawn", "AgentSpawn")) { exit 0 }

$sessionId = [string]$payload.session_id
if ([string]::IsNullOrWhiteSpace($sessionId) -or $sessionId.Length -gt 256) { exit 0 }

$herdr = if ([string]::IsNullOrWhiteSpace($env:HERDR_BIN_PATH)) { "herdr" } else { $env:HERDR_BIN_PATH }
$seq = [string][DateTimeOffset]::UtcNow.Ticks

$commandArgs = @(
    "pane", "report-agent-session", $env:HERDR_PANE_ID,
    "--source", "herdr:kiro", "--agent", "kiro-cli",
    "--agent-session-id", $sessionId,
    "--seq", $seq
)

$job = $null
try {
    $job = Start-Job -ScriptBlock {
        param($Executable, [object[]]$Arguments)
        & $Executable @Arguments *> $null
    } -ArgumentList $herdr, (,$commandArgs)
    if ($null -eq (Wait-Job -Job $job -Timeout 1)) {
        Stop-Job -Job $job
    }
} catch {
    $null = $_
} finally {
    if ($null -ne $job) {
        Remove-Job -Job $job -Force -ErrorAction SilentlyContinue
    }
}
