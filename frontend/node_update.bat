rem updates node and globally-installed tools

where nvm >NUL 2>&1
if '%errorlevel%' == '0' goto :use_nvm

set "command=powershell -NoProfile -ExecutionPolicy Bypass -File ..\helper_scripts\node_lts_install_or_update.ps1"
call %command% || goto :error

set "NODE_ENV_BAT=%LOCALAPPDATA%\fruitfacts\node\node_env.bat"
set "command=%NODE_ENV_BAT%"
call "%NODE_ENV_BAT%" || goto :error
goto :node_ready

:use_nvm
set "command=nvm install lts"
call %command% || goto :error

set "command=nvm use lts"
call %command% || goto :error

:node_ready

set "command=npm i -g npm-check-updates"
call %command% || goto :error

echo store versions to a file
echo | set /p dummy_name="node: " >node_versions.txt || goto :error
call node --version >>node_versions.txt || goto :error
echo | set /p dummy_name="npm-check-updates: " >>node_versions.txt || goto :error
call npm-check-updates --version >>node_versions.txt || goto :error

echo finished
goto :EOF

:error
echo %command% Failed with error #%errorlevel%
exit /b %errorlevel%
