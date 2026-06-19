set shell := ["pwsh.exe", "-c"]

build:
    cargo build --release -j 12
    Copy-Item target\release\*.exe G:\Dx\bin\ -Force -ErrorAction SilentlyContinue
