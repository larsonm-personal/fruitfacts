rem updates node and globally-installed tools

rem get admin for "nvm use..."
Net session >nul 2>&1 || (
    PowerShell start -verb runas '%~0'
    pause
    exit /b
)

cd /d "%~dp0"
set /p FRUITFACTS_NODE_VERSION=<..\.nvmrc
nvm install %FRUITFACTS_NODE_VERSION% || goto :error
nvm use %FRUITFACTS_NODE_VERSION% || goto :error
npm i -g npm-check-updates || goto :error
pause
goto :EOF

:error
echo Failed with error #%errorlevel%.
pause
exit /b %errorlevel%
