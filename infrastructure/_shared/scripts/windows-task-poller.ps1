<#
.SYNOPSIS
  Install / Uninstall / Status the KoproGo GitOps poller as a Windows Task Scheduler job.

.DESCRIPTION
  Equivalent of install-systemd-poller.sh for Windows hosts where systemd is
  unavailable. Creates a logon-triggered scheduled task that runs
  `gitops-deploy.sh watch` under Git Bash with TOPOLOGY=local ENV_NAME=env-dev.

  Tier 1 per CRITICAL.md: this script does not start anything autonomously —
  the user runs it explicitly with -Action Install (which only registers the
  task) then triggers it via the Task Scheduler UI or `Start-ScheduledTask`.

.PARAMETER Action
  Install   — register the scheduled task (does NOT start it)
  Uninstall — remove the scheduled task
  Status    — show task state + last run result
  Start     — start the task once (one-off, returns immediately)
  Stop      — stop the running task

.EXAMPLE
  .\infrastructure\_shared\scripts\windows-task-poller.ps1 -Action Install
  Start-ScheduledTask -TaskName 'KoproGo-GitOps-EnvDev'

.NOTES
  Requires: Git for Windows (provides bash.exe), Docker Desktop running.
  Runs as: current user (no admin elevation).
#>
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('Install', 'Uninstall', 'Status', 'Start', 'Stop')]
    [string]$Action
)

$ErrorActionPreference = 'Stop'
$TaskName = 'KoproGo-GitOps-EnvDev'
$RepoDir = (git rev-parse --show-toplevel) -replace '/', '\'
$BashExe = (Get-Command bash.exe -ErrorAction Stop).Source
$Script = "$RepoDir\infrastructure\_shared\scripts\gitops-deploy.sh"

function Test-Prereqs {
    if (-not (Test-Path "$RepoDir\.git")) {
        Write-Error "Not a git repo: $RepoDir"
    }
    if (-not (Test-Path $Script)) {
        Write-Error "gitops-deploy.sh not found at $Script"
    }
    if (-not (Test-Path "$RepoDir\infrastructure\monosite\local\env-dev\.env")) {
        Write-Warning ".env missing — copy from .env.example before starting the task."
    }
}

switch ($Action) {
    'Install' {
        Test-Prereqs

        # Action: bash.exe runs the watch loop with required env vars
        $bashCmd = "BRANCH=dev ENV_NAME=env-dev TOPOLOGY=local CHECK_INTERVAL=120 REPO_DIR='$RepoDir' '$Script' watch"
        $action = New-ScheduledTaskAction -Execute $BashExe -Argument "-l -c `"$bashCmd`"" -WorkingDirectory $RepoDir

        # Trigger: at user logon, no idle requirement
        $trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME

        # Settings: run on battery, no time limit, restart on failure
        $settings = New-ScheduledTaskSettingsSet `
            -AllowStartIfOnBatteries `
            -DontStopIfGoingOnBatteries `
            -ExecutionTimeLimit (New-TimeSpan -Days 365) `
            -RestartInterval (New-TimeSpan -Minutes 5) `
            -RestartCount 3 `
            -StartWhenAvailable

        # Principal: run as current user, do not store password (interactive)
        $principal = New-ScheduledTaskPrincipal -UserId "$env:USERDOMAIN\$env:USERNAME" -LogonType Interactive

        $task = New-ScheduledTask -Action $action -Trigger $trigger -Settings $settings -Principal $principal `
            -Description "KoproGo GitOps watcher — polls dev branch every 120s and redeploys local env-dev docker-compose"

        Register-ScheduledTask -TaskName $TaskName -InputObject $task -Force | Out-Null

        Write-Host "✅ Installed scheduled task: $TaskName" -ForegroundColor Green
        Write-Host ""
        Write-Host "Next steps (Tier 1 — humain valide):" -ForegroundColor Yellow
        Write-Host "  1. Copy infrastructure\monosite\local\env-dev\.env.example to .env"
        Write-Host "  2. Add to C:\Windows\System32\drivers\etc\hosts:"
        Write-Host "     127.0.0.1 envdev.koprogo.local api-envdev.koprogo.local"
        Write-Host "  3. Start-ScheduledTask -TaskName $TaskName"
        Write-Host "  4. Get-ScheduledTaskInfo -TaskName $TaskName"
    }

    'Uninstall' {
        Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
        Write-Host "✅ Removed scheduled task: $TaskName" -ForegroundColor Green
    }

    'Status' {
        $task = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
        if (-not $task) {
            Write-Host "Task not registered. Run with -Action Install." -ForegroundColor Yellow
            exit 1
        }
        $info = Get-ScheduledTaskInfo -TaskName $TaskName
        Write-Host "Task : $TaskName"
        Write-Host "State: $($task.State)"
        Write-Host "Last run    : $($info.LastRunTime)"
        Write-Host "Last result : $($info.LastTaskResult)"
        Write-Host "Next run    : $($info.NextRunTime)"
    }

    'Start' {
        Start-ScheduledTask -TaskName $TaskName
        Write-Host "✅ Started: $TaskName"
    }

    'Stop' {
        Stop-ScheduledTask -TaskName $TaskName
        Write-Host "✅ Stopped: $TaskName"
    }
}
