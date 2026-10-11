const
  UserPathScope = 0;
  SystemPathScope = 1;
  ExecutableInPathFalse = 0;
  ExecutableInPathTrue = 1;

function AddDirectoryToPathSetup(PathScope: Integer; DirectoryPath: PAnsiChar; ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'add_directory_to_path@files:inno_plugin.dll cdecl setuponly';

function RemoveDirectoryFromPathSetup(PathScope: Integer; DirectoryPath: PAnsiChar; ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'remove_directory_from_path@files:inno_plugin.dll cdecl setuponly';

function RemoveDirectoryFromPathUninstall(PathScope: Integer; DirectoryPath: PAnsiChar; ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'remove_directory_from_path@{app}\inno_plugin.dll cdecl uninstallonly';

function IsExecutableInPathSetup(ExecutableName: PAnsiChar; ExcludedDirectory: PAnsiChar; ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'is_executable_in_path@files:inno_plugin.dll cdecl setuponly';

function GetPathScope(): Integer;
begin
  if IsAdminInstallMode then
    Result := SystemPathScope
  else
    Result := UserPathScope;
end;

function TrimPathErrorBuffer(const ErrorBuffer: AnsiString): AnsiString;
var
  NullIndex: Integer;
begin
  NullIndex := Pos(#0, ErrorBuffer);
  if NullIndex = 0 then
    Result := ErrorBuffer
  else
    Result := Copy(ErrorBuffer, 1, NullIndex - 1);
end;

procedure HandlePathOperationResult(const ResultCode: Integer; const OperationName: String; const DirectoryPath: String; const ErrorBuffer: AnsiString; const RaiseOnError: Boolean);
var
  ErrorMessage: AnsiString;
  PromptMessage: String;
begin
  if ResultCode = 0 then
    exit;

  ErrorMessage := TrimPathErrorBuffer(ErrorBuffer);
  if ErrorMessage = '' then
    ErrorMessage := 'Unknown error.';

  if RaiseOnError then
    RaiseException(Format('Failed to %s "%s" in PATH: %s', [OperationName, DirectoryPath, String(ErrorMessage)]))
  else begin
    Log(Format('Failed to %s "%s" in PATH during uninstall: %s', [OperationName, DirectoryPath, String(ErrorMessage)]));
    PromptMessage := Format('Actiona could not %s "%s" in PATH during uninstall.' + #13#10 + #13#10 + 'You may need to update PATH manually.' + #13#10 + #13#10 + 'Error: %s', [OperationName, DirectoryPath, String(ErrorMessage)]);
    SuppressibleMsgBox(
      PromptMessage,
      mbError,
      MB_OK,
      IDOK
    );
  end;
end;

procedure ExecuteSetupPathOperation(const ShouldAdd: Boolean; const DirectoryPath: String);
var
  DirectoryPathAnsi: AnsiString;
  ErrorBuffer: AnsiString;
  ResultCode: Integer;
  OperationName: String;
begin
  DirectoryPathAnsi := AnsiString(DirectoryPath);
  SetLength(ErrorBuffer, 1024);
  ErrorBuffer[1] := #0;

  if ShouldAdd then begin
    OperationName := 'add';
    ResultCode := AddDirectoryToPathSetup(
      GetPathScope(),
      PAnsiChar(DirectoryPathAnsi),
      PAnsiChar(ErrorBuffer),
      Length(ErrorBuffer)
    );
  end else begin
    OperationName := 'remove';
    ResultCode := RemoveDirectoryFromPathSetup(
      GetPathScope(),
      PAnsiChar(DirectoryPathAnsi),
      PAnsiChar(ErrorBuffer),
      Length(ErrorBuffer)
    );
  end;

  HandlePathOperationResult(ResultCode, OperationName, DirectoryPath, ErrorBuffer, True);
end;

procedure ExecuteUninstallPathOperation(const DirectoryPath: String);
var
  DirectoryPathAnsi: AnsiString;
  ErrorBuffer: AnsiString;
  PluginDllPath: String;
  ResultCode: Integer;
begin
  DirectoryPathAnsi := AnsiString(DirectoryPath);
  SetLength(ErrorBuffer, 1024);
  ErrorBuffer[1] := #0;
  PluginDllPath := ExpandConstant('{app}\inno_plugin.dll');

  ResultCode := RemoveDirectoryFromPathUninstall(
    GetPathScope(),
    PAnsiChar(DirectoryPathAnsi),
    PAnsiChar(ErrorBuffer),
    Length(ErrorBuffer)
  );
  UnloadDLL(PluginDllPath);

  HandlePathOperationResult(ResultCode, 'remove', DirectoryPath, ErrorBuffer, False);
end;

function IsRunInPathElsewhere(const DirectoryPath: String): Boolean;
var
  ExecutableNameAnsi: AnsiString;
  DirectoryPathAnsi: AnsiString;
  ErrorBuffer: AnsiString;
  ResultCode: Integer;
begin
  ExecutableNameAnsi := 'actiona-run.exe';
  DirectoryPathAnsi := AnsiString(DirectoryPath);
  SetLength(ErrorBuffer, 1024);
  ErrorBuffer[1] := #0;

  ResultCode := IsExecutableInPathSetup(
    PAnsiChar(ExecutableNameAnsi),
    PAnsiChar(DirectoryPathAnsi),
    PAnsiChar(ErrorBuffer),
    Length(ErrorBuffer)
  );

  if ResultCode = ExecutableInPathTrue then
    Result := True
  else begin
    if ResultCode <> ExecutableInPathFalse then
      Log(Format('Failed to look for actiona-run.exe in PATH, adding "%s" anyway: %s', [DirectoryPath, String(TrimPathErrorBuffer(ErrorBuffer))]));
    Result := False;
  end;
end;

function ShouldAddToPath(): Boolean;
begin
  if not WizardIsTaskSelected('addtopath') then begin
    Result := False;
    exit;
  end;

#ifdef MyAddToPathOnlyIfRunMissing
  // Actiona Run already provides actiona-run on PATH; don't add a second copy.
  if IsRunInPathElsewhere(ExpandConstant('{app}')) then begin
    Log('actiona-run.exe is already in PATH; not adding the application directory.');
    Result := False;
    exit;
  end;
#endif

  Result := True;
end;
