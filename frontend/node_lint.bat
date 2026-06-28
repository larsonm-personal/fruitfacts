RMDIR /S /Q node_modules
del package-lock.json

set "command=ncu -u --reject eslint,@eslint/js"
call %command% || goto :error

echo "using --force because of react-debounce-input and react 19"

:: set variable to "npm install" so we can print that when there's an error
set "command=npm install --force"
call %command% || goto :error

set "command=npm run lint"
call %command% || goto :error

set "command=npm run build"
call %command% || goto :error

echo finished
goto :EOF

:error
echo %command% Failed with error #%errorlevel%
exit /b %errorlevel%
