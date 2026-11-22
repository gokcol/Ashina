; The Ashina installer for 64-bit Windows, built with Inno Setup 6 by the
; "Windows build" workflow:
;   ISCC /DAppVersion=2.2.2 /DSource=pack\Ashina /O<folder> /F<file name> MyPDF.iss
; It installs for the current user without asking for administrator rights
; (or for everyone, if chosen), adds a Start menu entry and an uninstaller,
; and, when ticked, lists Ashina in the "Open with" menu for PDF files.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef Source
  #define Source "..\..\pack\Ashina"
#endif
#ifndef AppArchitecture
  #define AppArchitecture "x64compatible"
#endif

[Setup]
AppId={{8C4E2F6A-5B1D-4C7E-9A2F-3D6B1E0F7A51}
AppName=Ashina
AppVersion={#AppVersion}
AppVerName=Ashina {#AppVersion}
AppPublisher=Ashina contributors
AppPublisherURL=https://github.com/gokcol/MyPDF
AppSupportURL=https://github.com/gokcol/MyPDF/issues
AppUpdatesURL=https://github.com/gokcol/MyPDF/releases
VersionInfoProductName=Ashina
VersionInfoProductVersion={#AppVersion}
VersionInfoVersion={#AppVersion}
VersionInfoDescription=Ashina installer
VersionInfoCopyright=© 2026 Ashina contributors · MIT OR Apache-2.0
DefaultDirName={autopf}\Ashina
UsePreviousAppDir=yes
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed={#AppArchitecture}
ArchitecturesInstallIn64BitMode={#AppArchitecture}
MinVersion=10.0
SetupIconFile=MyPDF.ico
UninstallDisplayIcon={app}\Ashina.exe
UninstallDisplayName=Ashina
InfoBeforeFile=before-install.txt
ChangesAssociations=yes
CloseApplications=yes
RestartApplications=no
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "turkish"; MessagesFile: "compiler:Languages\Turkish.isl"

[CustomMessages]
english.OpenWith=Add Ashina to the "Open with" menu for PDF files
turkish.OpenWith=Ashina’yı PDF dosyalarının "Birlikte aç" menüsüne ekle
english.Integration=Windows integration:
turkish.Integration=Windows ile bütünleşme:

[Tasks]
Name: "openwith"; Description: "{cm:OpenWith}"; GroupDescription: "{cm:Integration}"
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#Source}\Ashina.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Source}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Source}\FEATURES.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Source}\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Source}\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Source}\THIRD_PARTY_NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion
; The bridge statically links CTranslate2/SentencePiece and the MSVC runtime.
; These explicit entries make a missing runtime or its notices a build failure.
Source: "{#Source}\translation\mypdf_ct2.dll"; DestDir: "{app}\translation"; Flags: ignoreversion
Source: "{#Source}\translation\mypdf_ct2.dll.sha256"; DestDir: "{app}\translation"; Flags: ignoreversion
Source: "{#Source}\translation\licenses\*"; DestDir: "{app}\translation\licenses"; Flags: ignoreversion recursesubdirs createallsubdirs

; Offline OCR, including language models and dependency notices.
Source: "{#Source}\ocr\tesseract.exe"; DestDir: "{app}\ocr"; Flags: ignoreversion
Source: "{#Source}\ocr\*.txt"; DestDir: "{app}\ocr"; Flags: ignoreversion
Source: "{#Source}\ocr\*.json"; DestDir: "{app}\ocr"; Flags: ignoreversion
Source: "{#Source}\ocr\tessdata\*.traineddata"; DestDir: "{app}\ocr\tessdata"; Flags: ignoreversion
Source: "{#Source}\ocr\licenses\*"; DestDir: "{app}\ocr\licenses"; Flags: ignoreversion recursesubdirs createallsubdirs

; Replace the old executable and shortcuts during upgrade.
[InstallDelete]
Type: files; Name: "{app}\MyPDF.exe"; Check: IsLegacyInstall
Type: files; Name: "{autoprograms}\MyPDF.lnk"
Type: files; Name: "{autodesktop}\MyPDF.lnk"
Type: files; Name: "{autoprograms}\SkyPDF.lnk"
Type: files; Name: "{autodesktop}\SkyPDF.lnk"

[Icons]
Name: "{autoprograms}\Ashina"; Filename: "{app}\Ashina.exe"
Name: "{autodesktop}\Ashina"; Filename: "{app}\Ashina.exe"; Tasks: desktopicon

[Registry]
; Let ShellExecute locate this per-user or all-users installation by name.
Root: HKA; Subkey: "Software\Microsoft\Windows\CurrentVersion\App Paths\Ashina.exe"; ValueType: string; ValueName: ""; ValueData: "{app}\Ashina.exe"; Flags: uninsdeletekey
; "Open with" for PDF files. The default PDF app stays as it is, and
; uninstalling removes these entries again.
Root: HKA; Subkey: "Software\Classes\Applications\Ashina.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "Ashina"; Flags: uninsdeletekey; Tasks: openwith
Root: HKA; Subkey: "Software\Classes\Applications\Ashina.exe\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\Ashina.exe,0"; Tasks: openwith
Root: HKA; Subkey: "Software\Classes\Applications\Ashina.exe\SupportedTypes"; ValueType: string; ValueName: ".pdf"; ValueData: ""; Tasks: openwith
Root: HKA; Subkey: "Software\Classes\Applications\Ashina.exe\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Ashina.exe"" ""%1"""; Tasks: openwith
; Keep the existing PDF ProgID so Windows can still resolve saved defaults.
Root: HKA; Subkey: "Software\Classes\MyPDF.Document"; ValueType: string; ValueName: ""; ValueData: "PDF Document"; Flags: uninsdeletekey; Check: RegisterPdfAssociation
Root: HKA; Subkey: "Software\Classes\MyPDF.Document\Application"; ValueType: string; ValueName: "ApplicationName"; ValueData: "Ashina"; Check: RegisterPdfAssociation
Root: HKA; Subkey: "Software\Classes\MyPDF.Document\Application"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "Read, edit, annotate and sign PDF files on your computer."; Check: RegisterPdfAssociation
Root: HKA; Subkey: "Software\Classes\MyPDF.Document\Application"; ValueType: string; ValueName: "ApplicationCompany"; ValueData: "Ashina contributors"; Check: RegisterPdfAssociation
Root: HKA; Subkey: "Software\Classes\MyPDF.Document\Application"; ValueType: string; ValueName: "ApplicationIcon"; ValueData: "{app}\Ashina.exe,0"; Check: RegisterPdfAssociation
Root: HKA; Subkey: "Software\Classes\MyPDF.Document\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\Ashina.exe,0"; Check: RegisterPdfAssociation
Root: HKA; Subkey: "Software\Classes\MyPDF.Document\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Ashina.exe"" ""%1"""; Check: RegisterPdfAssociation
Root: HKA; Subkey: "Software\Classes\.pdf\OpenWithProgids"; ValueType: string; ValueName: "MyPDF.Document"; ValueData: ""; Flags: uninsdeletevalue; Check: RegisterPdfAssociation
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.pdf\OpenWithProgids"; ValueType: string; ValueName: "MyPDF.Document"; ValueData: ""; Flags: uninsdeletevalue; Check: RegisterPdfAssociation
; Windows Settings discovers desktop default-app candidates through these
; capabilities, in addition to the Shell's OpenWithProgids registration above.
Root: HKA; Subkey: "Software\MyPDF\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "Ashina"; Flags: uninsdeletekey; Check: RegisterPdfAssociation
Root: HKA; Subkey: "Software\MyPDF\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "Read, edit, annotate and sign PDF files on your computer."; Check: RegisterPdfAssociation
Root: HKA; Subkey: "Software\MyPDF\Capabilities"; ValueType: string; ValueName: "ApplicationIcon"; ValueData: "{app}\Ashina.exe,0"; Check: RegisterPdfAssociation
Root: HKA; Subkey: "Software\MyPDF\Capabilities\FileAssociations"; ValueType: string; ValueName: ".pdf"; ValueData: "MyPDF.Document"; Check: RegisterPdfAssociation
Root: HKA; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "Ashina"; ValueData: "Software\MyPDF\Capabilities"; Flags: uninsdeletevalue; Check: RegisterPdfAssociation

; Keep old shell links working on existing installs. Fresh installs only register Ashina.exe.
Root: HKA; Subkey: "Software\Microsoft\Windows\CurrentVersion\App Paths\MyPDF.exe"; ValueType: string; ValueName: ""; ValueData: "{app}\Ashina.exe"; Flags: uninsdeletekey; Check: IsLegacyInstall
Root: HKA; Subkey: "Software\Classes\Applications\MyPDF.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "Ashina"; Check: HasLegacyPdfHandler
Root: HKA; Subkey: "Software\Classes\Applications\MyPDF.exe"; ValueType: string; ValueName: "NoOpenWith"; ValueData: ""; Check: HasLegacyPdfHandler
Root: HKA; Subkey: "Software\Classes\Applications\MyPDF.exe\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\Ashina.exe,0"; Check: HasLegacyPdfHandler
Root: HKA; Subkey: "Software\Classes\Applications\MyPDF.exe\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Ashina.exe"" ""%1"""; Check: HasLegacyPdfHandler
Root: HKA; Subkey: "Software\RegisteredApplications"; ValueType: none; ValueName: "MyPDF"; Flags: deletevalue; Check: IsLegacyInstall

[Run]
Filename: "{app}\Ashina.exe"; Description: "{cm:LaunchProgram,Ashina}"; Flags: nowait postinstall skipifsilent

[Code]
function InstallRegistryRoot: Integer;
begin
  if IsAdminInstallMode then
    Result := HKLM
  else
    Result := HKCU;
end;

function IsLegacyInstall: Boolean;
var
  PreviousExe: String;
begin
  Result := RegQueryStringValue(InstallRegistryRoot,
    'Software\Microsoft\Windows\CurrentVersion\App Paths\MyPDF.exe', '', PreviousExe) and
    ((CompareText(PreviousExe, ExpandConstant('{app}\MyPDF.exe')) = 0) or
     (CompareText(PreviousExe, ExpandConstant('{app}\Ashina.exe')) = 0));
end;

function HasLegacyPdfHandler: Boolean;
begin
  Result := IsLegacyInstall and
    RegKeyExists(InstallRegistryRoot, 'Software\Classes\Applications\MyPDF.exe');
end;

function RegisterPdfAssociation: Boolean;
begin
  Result := WizardIsTaskSelected('openwith') or (IsLegacyInstall and
    RegKeyExists(InstallRegistryRoot, 'Software\Classes\MyPDF.Document'));
end;
