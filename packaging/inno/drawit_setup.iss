; DrawIt Inno Setup Script
; Configured for per-user installation (no admin rights required) and Microsoft Store Win32 compatibility

#define MyAppName "DrawIt"
#define MyAppVersion "0.3.0"
#define MyAppPublisher "Csuszy"
#define MyAppURL "https://github.com/csuszy/DrawIt"
#define MyAppExeName "DrawIt.exe"

[Setup]
AppId={{D37E56F1-3C3A-4F7F-9721-6A940F581C6E}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={localappdata}\Programs\{#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir=..\..\
OutputBaseFilename=DrawIt-Setup
SetupIconFile=..\..\DrawIt_logo.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\{#MyAppExeName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "hungarian"; MessagesFile: "compiler:Languages\Hungarian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "autostart"; Description: "DrawIt automatikus inditasa Windows bejelentkezeskor"; GroupDescription: "Egyeb beallitasok:"; Flags: unchecked

[Files]
Source: "..\..\DrawIt.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\DrawIt_logo.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\DrawIt_logo.svg"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.en.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\DrawIt_logo.ico"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\DrawIt_logo.ico"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "DrawIt"; ValueData: """{app}\{#MyAppExeName}"""; Tasks: autostart

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
