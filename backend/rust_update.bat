rem run after a rust version update to fetch it and update all tools

set "command=rustup update stable"
call %command% || goto :error

set "command=rustup toolchain install nightly"
call %command% || goto :error

set "command=rustup component add clippy"
call %command% || goto :error

set "command=cargo install cargo-edit --locked"
call %command% || goto :error

set "command=cargo install cargo-udeps --locked"
call %command% || goto :error

set "command=del Cargo.lock"
del Cargo.lock || goto :error

set "command=cargo test -- --include-ignored"
call %command% || goto :error

rem print versions of rust to a file ./rust_versions.txt

set "command=rustc --version > rust_versions.txt"
call %command% || goto :error

set "command=rustup --version >> rust_versions.txt"
call %command% || goto :error

set "command=cargo --version >> rust_versions.txt"
call %command% || goto :error

set "command=cargo clippy --version >> rust_versions.txt"
call %command% || goto :error

set "command=cargo udeps --version >> rust_versions.txt"
call %command% || goto :error

set "command=cargo upgrade --version >> rust_versions.txt"
call %command% || goto :error

goto :EOF

:error
echo %command% Failed with error #%errorlevel%
exit /b %errorlevel%
