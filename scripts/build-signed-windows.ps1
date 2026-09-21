[CmdletBinding()]
param(
    [ValidateSet('Local', 'Public')]
    [string] $Mode = 'Local'
)

$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$targetTriple = 'x86_64-pc-windows-msvc'
$tauriConfigPath = Join-Path $repoRoot 'src-tauri\tauri.conf.json'
$tauriConfig = Get-Content -LiteralPath $tauriConfigPath -Raw | ConvertFrom-Json
$releaseDir = Join-Path $repoRoot "src-tauri\target\$targetTriple\release"
$exePath = Join-Path $releaseDir 'my-ai-usage.exe'
$msiName = '{0}_{1}_x64_en-US.msi' -f $tauriConfig.productName, $tauriConfig.version
$msiPath = Join-Path $releaseDir "bundle\msi\$msiName"
$localSubject = 'CN=My AI Usage Local Code Signing'

function Find-SignTool {
    $sdkRoot = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
    $candidate = Get-ChildItem -LiteralPath $sdkRoot -Filter 'signtool.exe' -Recurse -File -ErrorAction SilentlyContinue |
        Where-Object { $_.Directory.Name -eq 'x64' } |
        Sort-Object FullName -Descending |
        Select-Object -First 1

    if (-not $candidate) {
        throw 'Windows SDK signtool.exe was not found under Program Files (x86)\Windows Kits\10\bin.'
    }

    return $candidate.FullName
}

function Find-CertificateByThumbprint([string] $Thumbprint) {
    $normalized = $Thumbprint -replace '\s', ''
    $certificate = Get-ChildItem -Path 'Cert:\CurrentUser\My' |
        Where-Object { $_.Thumbprint -eq $normalized -and $_.HasPrivateKey } |
        Select-Object -First 1

    if (-not $certificate) {
        throw "No private-key certificate with thumbprint $normalized exists in Cert:\CurrentUser\My."
    }

    return $certificate
}

function Ensure-LocalCertificate {
    $certificate = Get-ChildItem -Path 'Cert:\CurrentUser\My' |
        Where-Object {
            $_.Subject -eq $localSubject -and
            $_.HasPrivateKey -and
            $_.NotAfter -gt (Get-Date)
        } |
        Sort-Object NotAfter -Descending |
        Select-Object -First 1

    if (-not $certificate) {
        $certificate = New-SelfSignedCertificate `
            -Type CodeSigningCert `
            -Subject $localSubject `
            -CertStoreLocation 'Cert:\CurrentUser\My' `
            -HashAlgorithm SHA256 `
            -KeyAlgorithm RSA `
            -KeyLength 3072 `
            -NotAfter (Get-Date).AddYears(3)
    }

    $temporaryCertificate = Join-Path $env:TEMP "my-ai-usage-local-signing-$PID.cer"
    try {
        Export-Certificate -Cert $certificate -FilePath $temporaryCertificate | Out-Null

        foreach ($store in @('Cert:\CurrentUser\Root', 'Cert:\CurrentUser\TrustedPublisher')) {
            $trusted = Get-ChildItem -Path $store |
                Where-Object { $_.Thumbprint -eq $certificate.Thumbprint } |
                Select-Object -First 1

            if (-not $trusted) {
                Import-Certificate -FilePath $temporaryCertificate -CertStoreLocation $store | Out-Null
            }
        }
    }
    finally {
        Remove-Item -LiteralPath $temporaryCertificate -Force -ErrorAction SilentlyContinue
    }

    return $certificate
}

if ($Mode -eq 'Local') {
    $certificate = Ensure-LocalCertificate
}
else {
    if ([string]::IsNullOrWhiteSpace($env:MY_AI_USAGE_SIGNING_THUMBPRINT)) {
        throw 'Public mode requires MY_AI_USAGE_SIGNING_THUMBPRINT after the public certificate is imported into Cert:\CurrentUser\My.'
    }

    $certificate = Find-CertificateByThumbprint $env:MY_AI_USAGE_SIGNING_THUMBPRINT
}

if ($Mode -eq 'Public' -and [string]::IsNullOrWhiteSpace($env:MY_AI_USAGE_TIMESTAMP_URL)) {
    throw 'Public mode requires MY_AI_USAGE_TIMESTAMP_URL from the public certificate issuer.'
}

$signToolPath = Find-SignTool
$env:TAURI_WINDOWS_SIGNTOOL_PATH = $signToolPath
$windowsConfig = @{
    certificateThumbprint = $certificate.Thumbprint
    digestAlgorithm = 'sha256'
}

$signArguments = @(
    'sign'
    '/fd'
    'SHA256'
    '/sha1'
    $certificate.Thumbprint
)

if (-not [string]::IsNullOrWhiteSpace($env:MY_AI_USAGE_TIMESTAMP_URL)) {
    $windowsConfig.timestampUrl = $env:MY_AI_USAGE_TIMESTAMP_URL
    $signArguments += @('/tr', $env:MY_AI_USAGE_TIMESTAMP_URL, '/td', 'SHA256')
}

$windowsConfig.signCommand = @{
    cmd = $signToolPath
    args = $signArguments + @('%1')
}

$signingConfig = @{
    bundle = @{
        windows = $windowsConfig
    }
} | ConvertTo-Json -Compress -Depth 5

$temporaryTauriConfig = Join-Path $env:TEMP "my-ai-usage-tauri-signing-$PID.json"
Set-Content -LiteralPath $temporaryTauriConfig -Value $signingConfig -Encoding UTF8

try {
    Write-Host "Signing certificate: $($certificate.Thumbprint)"
    Write-Host "signtool: $signToolPath"

    & npm.cmd exec tauri -- build `
        --target $targetTriple `
        --bundles msi `
        --config $temporaryTauriConfig

    if ($LASTEXITCODE -ne 0) {
        throw "Tauri signed build failed with exit code $LASTEXITCODE."
    }

    foreach ($artifact in @($exePath, $msiPath)) {
        if (-not (Test-Path -LiteralPath $artifact)) {
            throw "Expected signed artifact was not produced: $artifact"
        }

        $signature = $null
        for ($attempt = 1; $attempt -le 12; $attempt++) {
            try {
                $signature = Get-AuthenticodeSignature -LiteralPath $artifact
                break
            }
            catch [System.UnauthorizedAccessException] {
                if ($attempt -eq 12) {
                    throw "Could not read signed artifact after waiting for a file lock. Close any security scan or installer using the file, restart Windows if needed, and rerun the build: $artifact"
                }

                Start-Sleep -Seconds 5
            }
        }

        if ($signature.Status -ne 'Valid') {
            throw "Signature validation failed for ${artifact}: $($signature.Status)"
        }

        if ($signature.SignerCertificate.Thumbprint -ne $certificate.Thumbprint) {
            throw "Unexpected signing certificate for $artifact."
        }

        Write-Host "Valid signature: $artifact"
    }
}
finally {
    Remove-Item -LiteralPath $temporaryTauriConfig -Force -ErrorAction SilentlyContinue
}
