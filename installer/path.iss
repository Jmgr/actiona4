const
  UserPathScope = 0;
  SystemPathScope = 1;

function AddDirectoryToPathSetup(PathScope: Integer; DirectoryPath: PAnsiChar; ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'add_directory_to_path@files:inno_plugin.dll cdecl setuponly';

function RemoveDirectoryFromPathSetup(PathScope: Integer; DirectoryPath: PAnsiChar; ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'remove_directory_from_path@files:inno_plugin.dll cdecl setuponly';

function RemoveDirectoryFromPathUninstall(PathScope: Integer; DirectoryPath: PAnsiChar; ErrorBuffer: PAnsiChar; ErrorBufferCapacity: Cardinal): Integer;
  external 'remove_directory_from_path@{app}\inno_plugin.dll cdecl uninstallonly';

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
