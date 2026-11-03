Set-PSReadLineOption -HistorySaveStyle SaveNothing -ErrorAction SilentlyContinue
$global:__opsisStarted = $false
function global:prompt {
  $code = if ($?) { 0 } elseif ($LASTEXITCODE) { $LASTEXITCODE } else { 1 }
  $e = [char]27
  $bel = [char]7
  $marks = ""
  if ($global:__opsisStarted) { $marks = "$e]133;D;$code$bel" }
  $global:__opsisStarted = $true
  $folder = (Get-Location).Path
  if ($folder.StartsWith($HOME)) { $folder = "~" + $folder.Substring($HOME.Length) }
  "$marks$e]133;A$bel$e[1;32mopsis$e[0m:$e[1;34m$folder$e[0m`$ $e]133;B$bel"
}
