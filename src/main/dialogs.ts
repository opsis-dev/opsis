import { dialog, type BrowserWindow, type OpenDialogOptions } from "electron";

export async function openDialog(
  win: BrowserWindow | null,
  options: OpenDialogOptions,
): Promise<string[]> {
  const res = win
    ? await dialog.showOpenDialog(win, options)
    : await dialog.showOpenDialog(options);
  return res.canceled ? [] : res.filePaths;
}
