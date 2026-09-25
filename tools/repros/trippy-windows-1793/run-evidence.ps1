<#
.SYNOPSIS
  C007 evidence controller for the Trippy #1793 Windows reproducer.

.DESCRIPTION
  Builds one isolated repro variant once, then runs its binary as a child
  process so an abort cannot kill this controller. Records per-run exit
  codes/stdout/stderr under logs/ and asserts the expected outcome family:
  Abort (baseline must demonstrate the defining C005/#1793 crash signature)
  or Clean (candidate must show zero aborts over bounded repeated runs).

  Must run in an elevated Windows process for the privileged UDP trace path.

.PARAMETER Variant
  One of: baseline, candidate-fix, candidate-master.

.PARAMETER Expect
  One of: Abort, Clean.

.PARAMETER Repeats
  Number of bounded loopback runs. Baseline typically 3 (abort on any run
  demonstrates the defect); candidates minimum 10.

.PARAMETER TimeoutSeconds
  Per-run wall-clock bound for one bounded trace invocation.
#>
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("baseline", "candidate-fix", "candidate-master")]
    [string]$Variant,

    [Parameter(Mandatory = $true)]
    [ValidateSet("Abort", "Clean")]
    [string]$Expect,

    [Parameter(Mandatory = $true)]
    [int]$Repeats,

    [int]$TimeoutSeconds = 120
)

$ErrorActionPreference = "Stop"

# Signed decimal for 0xC0000409 / STATUS_STACK_BUFFER_OVERRUN.
$AbortExitCode = -1073740791
# Defining C005/#1793 crash family in captured stderr.
$AbortSignature = "from_size_align_unchecked"

$HarnessRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$VariantDir = Join-Path $HarnessRoot $Variant
$LogsDir = Join-Path $HarnessRoot "logs"
New-Item -ItemType Directory -Force -Path $LogsDir | Out-Null

Write-Host "EVIDENCE variant=$Variant expect=$Expect repeats=$Repeats"
Write-Host "EVIDENCE rustc: $((rustc -Vv) -join ' | ')"
Write-Host "EVIDENCE cargo: $(cargo --version)"

$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
Write-Host "EVIDENCE elevated=$isAdmin"
if (-not $isAdmin) {
    Write-Error "EVIDENCE NOT_ELEVATED refusing privileged trace qualification"
    exit 2
}

Write-Host "EVIDENCE building variant $Variant ..."
cargo build --locked --manifest-path (Join-Path $VariantDir "Cargo.toml")
if ($LASTEXITCODE -ne 0) {
    Write-Error "EVIDENCE BUILD_FAILED variant=$Variant"
    exit 1
}

$BinaryName = "trippy-windows-1793-$Variant.exe"
$Binary = Join-Path $VariantDir ("target\debug\" + $BinaryName)
if (-not (Test-Path $Binary)) {
    Write-Error "EVIDENCE BINARY_MISSING path=$Binary"
    exit 1
}

$abortRuns = 0
$cleanRuns = 0
$otherRuns = 0
$timeoutRuns = 0
for ($i = 1; $i -le $Repeats; $i++) {
    $tag = "{0}-{1:00}" -f $Variant, $i
    $outFile = Join-Path $LogsDir "$tag.stdout.log"
    $errFile = Join-Path $LogsDir "$tag.stderr.log"
    Write-Host "EVIDENCE run $i/$Repeats ..."
    # Stream child output straight to files (never buffered in this
    # controller) so partial markers survive even a hung or aborted child;
    # the abort itself cannot kill this controller process.
    # NOTE: no `-Wait` here: the bounded wait below enforces the per-run
    # timeout, while `-Wait` would block unboundedly first.
    $child = Start-Process -FilePath $Binary -NoNewWindow -PassThru `
        -RedirectStandardOutput $outFile -RedirectStandardError $errFile
    # Per-run wall-clock bound on top of the trace's own bounded config, so
    # a hung child is recorded and killed instead of hanging the job. The
    # wait runs on the already-exited-or-running child handle; output is
    # already streaming to disk, so no pipe deadlock is possible.
    if (-not $child.WaitForExit($TimeoutSeconds * 1000)) {
        Stop-Process -InputObject $child -Force
        "EVIDENCE TIMEOUT after ${TimeoutSeconds}s" | Out-File -FilePath $errFile -Append -Encoding utf8
        Write-Host "EVIDENCE run $i TIMEOUT after ${TimeoutSeconds}s (recorded, continuing)"
        $timeoutRuns++
        continue
    }
    $exitCode = $child.ExitCode
    # Output already streamed to disk by the redirect above; read back only
    # for signature classification.
    [string]$stdout = ""
    [string]$stderr = ""
    if (Test-Path $outFile) {
        $stdout = [string](Get-Content -Raw $outFile)
    }
    if (Test-Path $errFile) {
        $stderr = [string](Get-Content -Raw $errFile)
    }
    if ($exitCode -eq 2) {
        Write-Error "EVIDENCE run $i NOT_ELEVATED (exit 2). The evidence host lost elevation; stop and reassess."
        exit 2
    }
    $aborted = ($exitCode -eq $AbortExitCode) -and ($stderr -match $AbortSignature)
    if ($exitCode -eq $AbortExitCode -and -not $aborted) {
        Write-Host "EVIDENCE run $i exit=$exitCode (abort code without family text; recording excerpt)"
        $otherRuns++
    }
    elseif ($aborted) {
        Write-Host "EVIDENCE run $i exit=$exitCode aborted=true"
        $abortRuns++
    }
    elseif ($exitCode -eq 0) {
        $result = ($stdout -split "`r?`n" | Select-String "REPRO RESULT" | Select-Object -First 1)
        Write-Host "EVIDENCE run $i exit=0 clean ($result)"
        $cleanRuns++
    }
    else {
        Write-Host "EVIDENCE run $i unexpected exit=$exitCode (non-abort, non-zero)"
        $otherRuns++
    }
}

Write-Host "EVIDENCE summary variant=$Variant abort_runs=$abortRuns clean_runs=$cleanRuns other_runs=$otherRuns timeout_runs=$timeoutRuns repeats=$Repeats"

if ($Expect -eq "Abort") {
    if ($abortRuns -ge 1) {
        Write-Host "EVIDENCE PASS baseline demonstrated the defining abort signature ($abortRuns/$Repeats)"
        exit 0
    }
    Write-Error "EVIDENCE FAIL baseline did not reproduce the defining abort signature (abort_runs=$abortRuns clean_runs=$cleanRuns other_runs=$otherRuns timeout_runs=$timeoutRuns). Stop and reassess per C007 stop conditions."
    exit 1
}
else {
    if ($abortRuns -eq 0 -and $otherRuns -eq 0 -and $timeoutRuns -eq 0 -and ($cleanRuns -eq $Repeats)) {
        Write-Host "EVIDENCE PASS candidate showed zero aborts over $Repeats bounded runs"
        exit 0
    }
    Write-Error "EVIDENCE FAIL candidate: abort_runs=$abortRuns other_runs=$otherRuns timeout_runs=$timeoutRuns over $Repeats runs. Classify as regression/distinct per C007 section 7."
    exit 1
}
