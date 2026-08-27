[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string] $TargetDir,
    [int] $TimeoutSeconds = 15
)

$ErrorActionPreference = 'Stop'

if (-not [Environment]::UserInteractive) {
    throw 'Windows singleton smoke test requires an interactive desktop session (the GitHub Windows runner provides one).'
}

$tools = @('xtools-time', 'xtools-json', 'xtools-trans')
$started = [System.Collections.Generic.List[System.Diagnostics.Process]]::new()
$bodyError = $null

function Wait-ForExit([System.Diagnostics.Process] $Process, [int] $Milliseconds) {
    if (-not $Process.WaitForExit($Milliseconds)) {
        $Process.Refresh()
        throw "Process $($Process.Id) did not exit within $Milliseconds ms."
    }
}

function Assert-Alive([System.Diagnostics.Process] $Process, [string] $Description) {
    $Process.Refresh()
    if ($Process.HasExited) {
        throw "$Description process $($Process.Id) exited unexpectedly (exit code $($Process.ExitCode))."
    }
}

try {
    foreach ($tool in $tools) {
        $path = Join-Path $TargetDir "$tool.exe"
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "Built tool not found: $path"
        }

        Write-Host "Testing singleton behavior for $tool"
        $first = Start-Process -FilePath $path -WorkingDirectory $TargetDir -PassThru
        $started.Add($first)
        Start-Sleep -Milliseconds 500
        Assert-Alive $first 'First'

        $second = Start-Process -FilePath $path -WorkingDirectory $TargetDir -PassThru
        $started.Add($second)
        Wait-ForExit $second ($TimeoutSeconds * 1000)
        if ($second.ExitCode -ne 0) {
            throw "Second $tool launch exited with code $($second.ExitCode), expected a normal singleton handoff."
        }
        Assert-Alive $first 'First'

        $first.CloseMainWindow() | Out-Null
        if (-not $first.WaitForExit(3000)) {
            $first.Kill()
            [void]$first.WaitForExit(3000)
        }
        if (-not $first.HasExited) {
            throw "Could not terminate first $tool process $($first.Id)."
        }

        $third = Start-Process -FilePath $path -WorkingDirectory $TargetDir -PassThru
        $started.Add($third)
        Start-Sleep -Milliseconds 500
        Assert-Alive $third 'Later launch'
        Write-Host "$tool singleton smoke test passed"
    }
}
catch {
    $bodyError = $_
}
finally {
    $cleanupErrors = [System.Collections.Generic.List[System.Exception]]::new()
    foreach ($process in $started) {
        try {
            $process.Refresh()
            if (-not $process.HasExited) {
                $process.Kill()
                Wait-ForExit $process 3000
            }
        } catch {
            $cleanupErrors.Add([System.Exception]::new(
                "Cleanup failed for process $($process.Id): $($_.Exception.Message)",
                $_.Exception
            ))
        }
    }

    if ($cleanupErrors.Count -gt 0) {
        if ($null -ne $bodyError) {
            $cleanupErrors.Insert(0, $bodyError.Exception)
        }
        throw [System.AggregateException]::new('Windows singleton smoke test or process cleanup failed.', $cleanupErrors)
    }
}

if ($null -ne $bodyError) {
    throw $bodyError
}
