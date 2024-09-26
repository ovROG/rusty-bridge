cargo build -r --workspace 

xcopy %~dp0target\release\rusty-bridge.exe %~dp0target\bundle\rusty-bridge.exe /Y
xcopy %~dp0target\release\rusty-bridge-ui.exe %~dp0target\bundle\rusty-bridge-ui.exe /Y
xcopy %~dp0firewall.bat %~dp0target\bundle\firewall.bat /Y
xcopy %~dp0readme.md %~dp0target\bundle\readme.md /Y