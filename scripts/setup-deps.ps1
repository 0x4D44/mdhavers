# setup-deps.ps1 - Install mdhavers optional dependencies on Windows
#
# Usage: .\scripts\setup-deps.ps1 [OPTIONS]
#
# Options:
#   -All       Install all optional dependencies (default)
#   -LLVM      Install LLVM 15
#   -Graphics  Install graphics dependencies (CMake)
#   -Help      Show this help message
#
# Requires: Administrator privileges for some operations

param(
    [switch]$All,
    [switch]$LLVM,
    [switch]$Graphics,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# LLVM version to install
$LLVM_VERSION = "15.0.7"
$LLVM_INSTALL_DIR = "C:\Program Files\LLVM"

function Write-Info($message) {
    Write-Host "[INFO] " -ForegroundColor Blue -NoNewline
    Write-Host $message
}

function Write-Success($message) {
    Write-Host "[OK] " -ForegroundColor Green -NoNewline
    Write-Host $message
}

function Write-Warning-Custom($message) {
    Write-Host "[WARN] " -ForegroundColor Yellow -NoNewline
    Write-Host $message
}

function Write-Error-Custom($message) {
    Write-Host "[ERROR] " -ForegroundColor Red -NoNewline
    Write-Host $message
    exit 1
}

function Test-Admin {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Test-Chocolatey {
    return (Get-Command choco -ErrorAction SilentlyContinue) -ne $null
}

function Test-Winget {
    return (Get-Command winget -ErrorAction SilentlyContinue) -ne $null
}

function Install-Chocolatey {
    Write-Info "Installing Chocolatey..."
    Set-ExecutionPolicy Bypass -Scope Process -Force
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
    Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    Write-Success "Chocolatey installed"
}

function Install-LLVM {
    Write-Info "Installing LLVM $LLVM_VERSION..."

    # Check if already installed
    if (Test-Path "$LLVM_INSTALL_DIR\bin\clang.exe") {
        $version = & "$LLVM_INSTALL_DIR\bin\clang.exe" --version 2>$null | Select-String -Pattern "(\d+\.\d+\.\d+)" | ForEach-Object { $_.Matches[0].Value }
        if ($version -like "15.*") {
            Write-Success "LLVM 15 already installed at $LLVM_INSTALL_DIR"
            Set-LLVMEnvironment
            return
        }
    }

    # Try winget first
    if (Test-Winget) {
        Write-Info "Attempting installation via winget..."
        try {
            winget install LLVM.LLVM --version $LLVM_VERSION --accept-package-agreements --accept-source-agreements
            Write-Success "LLVM installed via winget"
            Set-LLVMEnvironment
            return
        } catch {
            Write-Warning-Custom "winget installation failed, trying alternative methods..."
        }
    }

    # Try chocolatey
    if (Test-Chocolatey) {
        Write-Info "Attempting installation via Chocolatey..."
        try {
            choco install llvm --version=$LLVM_VERSION -y
            Write-Success "LLVM installed via Chocolatey"
            Set-LLVMEnvironment
            return
        } catch {
            Write-Warning-Custom "Chocolatey installation failed, trying direct download..."
        }
    }

    # Direct download as fallback
    Write-Info "Downloading LLVM directly from releases.llvm.org..."
    $downloadUrl = "https://github.com/llvm/llvm-project/releases/download/llvmorg-$LLVM_VERSION/LLVM-$LLVM_VERSION-win64.exe"
    $installerPath = "$env:TEMP\LLVM-$LLVM_VERSION-win64.exe"

    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $installerPath -UseBasicParsing
        Write-Info "Running LLVM installer (silent mode)..."
        Start-Process -FilePath $installerPath -ArgumentList "/S" -Wait
        Remove-Item $installerPath -Force -ErrorAction SilentlyContinue
        Write-Success "LLVM installed via direct download"
        Set-LLVMEnvironment
    } catch {
        Write-Error-Custom "Failed to download/install LLVM. Please install manually from: https://releases.llvm.org/download.html"
    }
}

function Set-LLVMEnvironment {
    Write-Info "Setting LLVM environment variables..."

    # Set LLVM_SYS_150_PREFIX for inkwell crate
    $llvmPath = $LLVM_INSTALL_DIR
    if (-not (Test-Path $llvmPath)) {
        # Try to find LLVM in common locations
        $possiblePaths = @(
            "C:\Program Files\LLVM",
            "C:\LLVM",
            "$env:LOCALAPPDATA\Programs\LLVM"
        )
        foreach ($path in $possiblePaths) {
            if (Test-Path "$path\bin\clang.exe") {
                $llvmPath = $path
                break
            }
        }
    }

    if (Test-Path $llvmPath) {
        # Set for current session
        $env:LLVM_SYS_150_PREFIX = $llvmPath
        $env:PATH = "$llvmPath\bin;$env:PATH"

        # Set persistently for user
        [Environment]::SetEnvironmentVariable("LLVM_SYS_150_PREFIX", $llvmPath, "User")

        # Add to PATH if not already there
        $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        if ($currentPath -notlike "*$llvmPath\bin*") {
            [Environment]::SetEnvironmentVariable("PATH", "$llvmPath\bin;$currentPath", "User")
        }

        Write-Success "LLVM_SYS_150_PREFIX set to: $llvmPath"
        Write-Info "You may need to restart your terminal for PATH changes to take effect"
    } else {
        Write-Warning-Custom "Could not find LLVM installation. Please set LLVM_SYS_150_PREFIX manually."
    }
}

function Install-Graphics {
    Write-Info "Installing graphics dependencies (CMake)..."

    # CMake is the main dependency for building raylib on Windows
    if (Get-Command cmake -ErrorAction SilentlyContinue) {
        Write-Success "CMake already installed: $(cmake --version | Select-Object -First 1)"
        return
    }

    if (Test-Winget) {
        Write-Info "Installing CMake via winget..."
        winget install Kitware.CMake --accept-package-agreements --accept-source-agreements
        Write-Success "CMake installed via winget"
    } elseif (Test-Chocolatey) {
        Write-Info "Installing CMake via Chocolatey..."
        choco install cmake -y
        Write-Success "CMake installed via Chocolatey"
    } else {
        Write-Warning-Custom "Please install CMake manually from: https://cmake.org/download/"
    }
}

function Test-VisualStudio {
    # Check for Visual Studio Build Tools
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWhere) {
        $vsPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($vsPath) {
            Write-Success "Visual Studio Build Tools found at: $vsPath"
            return $true
        }
    }

    # Check for standalone Build Tools
    $buildToolsPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools"
    if (Test-Path $buildToolsPath) {
        Write-Success "Visual Studio Build Tools 2022 found"
        return $true
    }

    return $false
}

function Install-BuildTools {
    Write-Info "Checking for Visual Studio Build Tools..."

    if (Test-VisualStudio) {
        return
    }

    Write-Warning-Custom "Visual Studio Build Tools not found."
    Write-Info "Rust on Windows requires Visual Studio Build Tools with C++ workload."
    Write-Info ""
    Write-Info "Install options:"
    Write-Info "  1. Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/"
    Write-Info "  2. Or via winget: winget install Microsoft.VisualStudio.2022.BuildTools --override '--add Microsoft.VisualStudio.Workload.VCTools --passive'"
    Write-Info "  3. Or via Chocolatey: choco install visualstudio2022buildtools --package-parameters '--add Microsoft.VisualStudio.Workload.VCTools'"
    Write-Info ""

    $response = Read-Host "Would you like to install Build Tools now? (y/N)"
    if ($response -eq 'y' -or $response -eq 'Y') {
        if (Test-Winget) {
            Write-Info "Installing via winget..."
            winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --passive"
        } elseif (Test-Chocolatey) {
            Write-Info "Installing via Chocolatey..."
            choco install visualstudio2022buildtools --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools" -y
        } else {
            Write-Warning-Custom "Neither winget nor Chocolatey available. Please install manually."
        }
    }
}

function Test-Rust {
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Success "Rust found: $(rustc --version)"
        return $true
    }
    Write-Warning-Custom "Rust not found. Install from: https://rustup.rs/"
    return $false
}

function Show-Help {
    Write-Host "setup-deps.ps1 - Install mdhavers optional dependencies on Windows"
    Write-Host ""
    Write-Host "Usage: .\scripts\setup-deps.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -All       Install all optional dependencies (default)"
    Write-Host "  -LLVM      Install LLVM 15"
    Write-Host "  -Graphics  Install graphics dependencies (CMake)"
    Write-Host "  -Help      Show this help message"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\scripts\setup-deps.ps1             # Install everything"
    Write-Host "  .\scripts\setup-deps.ps1 -LLVM       # Install only LLVM"
    Write-Host "  .\scripts\setup-deps.ps1 -Graphics   # Install only graphics deps"
    Write-Host ""
    Write-Host "Note: Some operations require administrator privileges."
}

# Main
function Main {
    Write-Host "==========================================="
    Write-Host "  mdhavers Dependency Installer (Windows)  "
    Write-Host "==========================================="
    Write-Host ""

    if ($Help) {
        Show-Help
        return
    }

    # If no specific flags, install all
    if (-not $LLVM -and -not $Graphics) {
        $All = $true
    }

    # Check admin status
    if (-not (Test-Admin)) {
        Write-Warning-Custom "Not running as Administrator. Some installations may fail."
        Write-Info "Consider running: Start-Process powershell -Verb RunAs -ArgumentList '-File', '$PSCommandPath'"
        Write-Host ""
    }

    # Check Rust
    Test-Rust | Out-Null

    # Check/install Build Tools
    Install-BuildTools

    # Install requested components
    if ($All -or $LLVM) {
        Install-LLVM
    }

    if ($All -or $Graphics) {
        Install-Graphics
    }

    Write-Host ""
    Write-Host "==========================================="
    Write-Success "Dependency installation complete!"
    Write-Host "==========================================="
    Write-Host ""
    Write-Info "Next steps:"
    Write-Host "  1. Restart your terminal (for PATH changes)"
    Write-Host "  2. Build with all features: cargo build"
    Write-Host "  3. Or build minimal: cargo build --no-default-features --features minimal"
    Write-Host "  4. Run tests: cargo test"
    Write-Host ""

    # Verify LLVM setup
    if ($All -or $LLVM) {
        if ($env:LLVM_SYS_150_PREFIX) {
            Write-Success "LLVM_SYS_150_PREFIX = $env:LLVM_SYS_150_PREFIX"
        } else {
            Write-Warning-Custom "LLVM_SYS_150_PREFIX not set. You may need to set it manually."
        }
    }
}

Main
