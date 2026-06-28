set "command=del Cargo.lock"
del Cargo.lock || goto :error

set "command=cargo update"
call %command% || goto :error

set "command=cargo upgrade --pinned --recursive false --exclude libsqlite3-sys"
call %command% || goto :error

set "command=cargo fmt"
call %command% || goto :error

set "command=cargo fix --allow-dirty"
call %command% || goto :error

set "command=cargo clippy --fix --allow-dirty"
call %command% || goto :error

set "command=cargo fmt"
call %command% || goto :error

set "command=cargo build"
call %command% || goto :error

set "command=cargo test -- --include-ignored"
call %command% || goto :error

goto :EOF

:error
echo %command% Failed with error #%errorlevel%
exit /b %errorlevel%
