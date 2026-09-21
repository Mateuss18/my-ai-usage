# Windows Code Signing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate a locally trusted signed Windows MSI and executable now, while keeping the same build flow ready for a public Authenticode certificate later.

**Architecture:** A PowerShell release script owns certificate discovery/creation, temporary Tauri signing configuration, the x64 MSI build, and signature verification. The certificate thumbprint is passed through a temporary config file instead of being committed to `tauri.conf.json`; Tauri invokes the Windows SDK `signtool` during bundling. Public CI reuses the script with a certificate already imported into the current-user certificate store.

**Tech Stack:** PowerShell, Windows certificate stores, Windows 11 SDK `signtool.exe`, Tauri CLI 2.11.4, npm scripts, Authenticode.

**Spec:** `docs/superpowers/specs/2026-09-10-windows-code-signing-design.md`

## Global Constraints

- The local certificate is only for testing on this Windows machine and is not a public trust solution.
- The private key must remain outside Git.
- The executable and MSI must both finish with `Get-AuthenticodeSignature` status `Valid` on the trusted local machine.
- The release command must build the existing x64 MSI and must not change the Tauri/Vue application architecture.
- End users must not be instructed to disable antivirus protection.

---

### Task 1: Add the Windows signed-build script

**Files:**
- Create: `scripts/build-signed-windows.ps1`

**Interfaces:**
- Consumes: `-Mode Local` or `-Mode Public`; `MY_AI_USAGE_SIGNING_THUMBPRINT` for public mode; optional `MY_AI_USAGE_TIMESTAMP_URL` for a public timestamp server.
- Produces: a signed `my-ai-usage.exe` and signed `My AI Usage_<version>_x64_en-US.msi` under `src-tauri/target/x86_64-pc-windows-msvc/release`.

- [ ] **Step 1: Add the script with explicit local/public modes**

```powershell
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

if ($Mode -eq 'Public' -and [string]::IsNullOrWhiteSpace($env:MY_AI_USAGE_TIMESTAMP_URL)) {
    throw 'Public mode requires MY_AI_USAGE_TIMESTAMP_URL from the public certificate issuer.'
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
```

- [ ] **Step 2: Run a syntax-only PowerShell parse check**

Run:

```powershell
rtk powershell.exe -NoProfile -Command "[System.Management.Automation.Language.Parser]::ParseFile('scripts/build-signed-windows.ps1', [ref]$null, [ref]$null) | Out-Null; Write-Output 'PowerShell syntax OK'"
```

Expected: `PowerShell syntax OK` with no parser error.

- [ ] **Step 3: Verify the script changes only the current-user certificate stores**

Run:

```powershell
rtk rg -n "Cert:\\CurrentUser|Export-Certificate|Import-Certificate|MY_AI_USAGE_SIGNING_THUMBPRINT|TAURI_WINDOWS_SIGNTOOL_PATH" scripts/build-signed-windows.ps1
```

Expected: the script contains no private-key file, password literal, repository certificate, or machine-wide certificate-store path.

### Task 2: Expose signed build commands and document release behavior

**Files:**
- Modify: `package.json:6-14`
- Modify: `README.md:32-42`

**Interfaces:**
- Consumes: `scripts/build-signed-windows.ps1` from Task 1.
- Produces: `npm.cmd run build:windows:signed:local` for this PC and `npm.cmd run build:windows:signed` for a public certificate imported by CI or a release machine.

- [ ] **Step 1: Add the two Windows signing scripts to `package.json`**

Add these entries to the existing `scripts` object without changing the existing `build`, `build:frontend`, or `build:tauri` commands:

```json
"build:windows:signed:local": "powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/build-signed-windows.ps1 -Mode Local",
"build:windows:signed": "powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/build-signed-windows.ps1 -Mode Public"
```

- [ ] **Step 2: Replace the unsigned-only installation guidance with signed-build instructions**

Document these exact commands in the `Distribuicao e instalacao` section:

```powershell
npm.cmd run build:windows:signed:local
```

Explain that this command creates/trusts a certificate named `My AI Usage Local Code Signing` only for the current Windows user, builds the x64 MSI, and verifies the `.exe` and `.msi`. State that this local certificate is not suitable for public distribution.

Document that the public release command is `npm.cmd run build:windows:signed` after the release machine/CI imports the public certificate into `Cert:\CurrentUser\My`, sets `MY_AI_USAGE_SIGNING_THUMBPRINT` to that certificate's SHA-1 thumbprint, and sets `MY_AI_USAGE_TIMESTAMP_URL` to the timestamp service supplied by the certificate issuer. The private key must be stored as a CI secret or protected certificate-store key, and only artifacts whose `Get-AuthenticodeSignature` status is `Valid` may be published. The documentation must explicitly say that end users should not disable antivirus protection.

- [ ] **Step 3: Check the documentation and package JSON for valid formatting**

Run:

```powershell
rtk npm.cmd pkg get scripts.build:windows:signed:local scripts.build:windows:signed
rtk git diff --check
```

Expected: both npm scripts print their PowerShell commands and `git diff --check` prints no errors.

### Task 3: Create the local certificate, build, and verify the signed artifacts

**Files:**
- Runtime-only: `Cert:\CurrentUser\My`, `Cert:\CurrentUser\Root`, `Cert:\CurrentUser\TrustedPublisher`
- Generated/ignored: `src-tauri/target/x86_64-pc-windows-msvc/release/my-ai-usage.exe` and `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/My AI Usage_0.1.0_x64_en-US.msi`

**Interfaces:**
- Consumes: `npm.cmd run build:windows:signed:local` from Task 2.
- Produces: local certificate thumbprint, signed x64 executable, signed x64 MSI, and verification output.

- [ ] **Step 1: Run the local signed build**

Run:

```powershell
rtk npm.cmd run build:windows:signed:local
```

Expected: the frontend check/build passes, Tauri invokes the Windows SDK `signtool.exe`, and the script prints `Valid signature` for both the executable and MSI.

- [ ] **Step 2: Independently verify certificate trust and both artifact signatures**

Run:

```powershell
rtk powershell.exe -NoProfile -Command "Get-ChildItem Cert:\CurrentUser\My,Cert:\CurrentUser\Root,Cert:\CurrentUser\TrustedPublisher | Where-Object Subject -eq 'CN=My AI Usage Local Code Signing' | Select-Object Subject,Thumbprint,NotAfter; Get-AuthenticodeSignature 'src-tauri\target\x86_64-pc-windows-msvc\release\my-ai-usage.exe' | Select-Object Status,SignerCertificate; Get-AuthenticodeSignature 'src-tauri\target\x86_64-pc-windows-msvc\release\bundle\msi\My AI Usage_0.1.0_x64_en-US.msi' | Select-Object Status,SignerCertificate"
```

Expected: the certificate is present in all three current-user stores and both signatures report `Status` as `Valid`.

- [ ] **Step 3: Validate the MSI database and preserve unrelated work**

Run the existing MSI database smoke check against the generated MSI, then run:

```powershell
rtk git status --short --branch
rtk git diff --check
```

Expected: the MSI opens as a Windows Installer database with product `My AI Usage` version `0.1.0`; only the signing script, package/README changes, and approved spec/plan are new, while the pre-existing `src-tauri/Cargo.toml` status is preserved.
