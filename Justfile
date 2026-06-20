set shell := ["pwsh.exe", "-c"]

build:
    cargo build --release -j 12
    New-Item -ItemType Directory -Force -Path G:\Dx\bin | Out-Null
    Copy-Item target\release\*.exe G:\Dx\bin\ -Force -ErrorAction SilentlyContinue

