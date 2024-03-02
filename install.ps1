param(
    $abi = "msvc",
    $arch = "x86_64"
)

$release = "0.1.0-alpha.2"
$os = "windows"

$base_dir = "$HOME\.sys-kaleido"
$dest_file = "${base_dir}\downloads\sys-kaleido${release}.${os}-${arch}.exe"
$url = "https://github.com/jinyuli/sys-kaleido/releases/download/v${release}/sys-kaleido-${arch}-pc-${os}-${abi}.exe"

function sys-kaleido-New-Dirs {
    New-Item -Force -Path "$base_dir\downloads", "$base_dir\bin" -ItemType "directory"
}

function sys-kaleido-Clean-Dirs {
    Remove-Item -Recurse -Path "$base_dir\downloads"
}

function sys-kaleido-Download-Release {
    $StatusCode = 400
    try {
        Invoke-WebRequest -Uri "$url" -OutFile "$dest_file"
        $StatusCode = 200
    } catch {
        Write-Error "Failed to download file: $PSItem"
    }
    return $StatusCode
}

function sys-kaleido-Install {
    Copy-Item "$dest_file" -Destination "$base_dir\bin\sys-kaleido.exe"
}

function sys-kaleido-Set-Path {
    $paths = [System.Environment]::GetEnvironmentVariable("PATH", [System.EnvironmentVariableTarget]::User) -split ';'
    $newPaths = @("$base_dir\bin")

    foreach ($p in $newPaths) {
        if ($p -in $paths) {
            Write-Output "$p already exists"
            continue
        }

        [System.Environment]::SetEnvironmentVariable(
            "PATH",
            [System.Environment]::GetEnvironmentVariable("PATH", [System.EnvironmentVariableTarget]::User) + "$p;",
            [System.EnvironmentVariableTarget]::User
        )
        Write-Host -ForegroundColor Green "$p appended"
    }
}

Write-Host -ForegroundColor Blue "[1/3] Downloading ${url}"
sys-kaleido-New-Dirs

$StatusCode = sys-kaleido-Download-Release

if ($StatusCode -eq 200) {
    Write-Host -ForegroundColor Blue "[2/3] Install sys-kaleido to the ${base_dir}\bin"
    sys-kaleido-Install

    Write-Host -ForegroundColor Blue "[3/3] Set environment variables"
    sys-kaleido-Set-Path

    Write-Host -ForegroundColor Green "sys-kaleido $release installed!"
}

sys-kaleido-Clean-Dirs