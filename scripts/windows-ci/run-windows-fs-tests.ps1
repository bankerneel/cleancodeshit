[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $TestExecutable
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
    throw 'This helper requires a disposable GitHub-hosted runner.'
}

function Invoke-LibTest {
    param(
        [string] $Executable,
        [string[]] $Arguments,
        [string[]] $ExpectedTest,
        [pscredential] $Credential
    )

    $start = [System.Diagnostics.ProcessStartInfo]::new($Executable)
    $start.WorkingDirectory = $stage
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    foreach ($argument in $Arguments) {
        $start.ArgumentList.Add($argument)
    }
    if ($Credential) {
        $start.UserName = $Credential.UserName
        $start.Domain = '.'
        $start.Password = $Credential.Password
        $start.LoadUserProfile = $true
    }

    $process = [System.Diagnostics.Process]::Start($start)
    try {
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit(300000)) {
            $process.Kill($true)
            $process.WaitForExit()
            throw 'The Windows filesystem tests timed out.'
        }
        Write-Host ($stdout.GetAwaiter().GetResult())
        Write-Host ($stderr.GetAwaiter().GetResult())
        if ($process.ExitCode -ne 0) {
            throw "Windows filesystem tests exited with status $($process.ExitCode)."
        }
        foreach ($test in $ExpectedTest) {
            $passed = 'test ' + [regex]::Escape($test) + ' \.\.\. ok'
            if ($stdout.Result -notmatch $passed) {
                throw "Windows filesystem tests did not pass $test."
            }
        }
    }
    finally {
        $process.Dispose()
    }
}

$principal = [Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'The original runner token must be elevated for the refusal fixture.'
}

$elevatedTest = 'windows_fs::tests_policy::refuses_elevated_process'
$standardTests = @(
    'windows_fs::tests_policy::allows_standard_process'
    'windows_fs::tests_handles::file_symlink_is_skipped_and_refused'
)
$source = (Resolve-Path -LiteralPath $TestExecutable).Path
$stage = Join-Path $env:ProgramData ('ccs-windows-fs-' + [guid]::NewGuid().ToString('N'))
$executable = Join-Path $stage 'ccs-core-tests.exe'
$links = Join-Path $stage 'ccs-ci-links'
$target = Join-Path $links 'target.txt'
$link = Join-Path $links 'link.txt'
$name = 'ccs-ci-' + [guid]::NewGuid().ToString('N').Substring(0, 12)
$password = ConvertTo-SecureString ('Cc1!' + [guid]::NewGuid().ToString('N')) -AsPlainText -Force
$account = $null

try {
    New-Item -ItemType Directory -Path $stage | Out-Null
    Invoke-LibTest -Executable $source -Arguments @('--ignored', '--exact', $elevatedTest) -ExpectedTest $elevatedTest

    $account = New-LocalUser -Name $name -Password $password
    $users = [Security.Principal.SecurityIdentifier]::new('S-1-5-32-545')
    if (-not (Get-LocalGroupMember -SID $users | Where-Object SID -eq $account.SID)) {
        Add-LocalGroupMember -SID $users -Member $account
    }
    $credential = [pscredential]::new($name, $password)
    Copy-Item -LiteralPath $source -Destination $executable
    New-Item -ItemType Directory -Path $links | Out-Null
    Set-Content -LiteralPath $target -Value 'symlink target'
    New-Item -ItemType SymbolicLink -Path $link -Target $target | Out-Null

    # The test account can read the staged harness and link fixture. Its own
    # loaded profile provides writable temporary directories for all mutations.
    & icacls.exe $stage /inheritance:r /grant:r '*S-1-5-32-544:(OI)(CI)(F)' '*S-1-5-18:(OI)(CI)(F)' "*$($account.SID):(OI)(CI)(RX)"
    if ($LASTEXITCODE -ne 0) {
        throw 'Could not set the ACL on the disposable test staging directory.'
    }

    Invoke-LibTest -Executable $executable -Arguments @('windows_fs::', '--include-ignored', '--skip', $elevatedTest) -ExpectedTest $standardTests -Credential $credential
}
finally {
    try {
        if ($account) {
            Get-CimInstance Win32_UserProfile -Filter "SID='$($account.SID)'" | Remove-CimInstance
        }
    }
    finally {
        if ($account) {
            Remove-LocalUser -SID $account.SID
        }
        if ([System.IO.Directory]::Exists($links)) {
            [System.IO.File]::Delete($link)
            [System.IO.File]::Delete($target)
            [System.IO.Directory]::Delete($links)
        }
        if ([System.IO.Directory]::Exists($stage)) {
            [System.IO.File]::Delete($executable)
            [System.IO.Directory]::Delete($stage)
        }
        $password.Dispose()
    }
}
