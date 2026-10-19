import { customType } from "drizzle-orm/sqlite-core";
import { safeStorage } from "electron";

function requireSecureStorage(): void {
  if (!safeStorage.isEncryptionAvailable())
    throw new Error("This computer has no secure storage available, so keys can't be saved.");
  if (process.platform === "linux" && safeStorage.getSelectedStorageBackend() === "basic_text")
    throw new Error(
      "This computer has no keyring to protect keys, so they can't be saved. Set up a keyring and try again.",
    );
}

function seal(value: string): string {
  requireSecureStorage();
  return safeStorage.encryptString(value).toString("base64");
}

function unseal(sealed: string): string {
  try {
    return safeStorage.decryptString(Buffer.from(sealed, "base64"));
  } catch {
    throw new Error("This key can't be read on this computer. Enter it again.");
  }
}

// TODO: To be used later
export const encryptedText = customType<{ data: string; driverData: string }>({
  dataType: () => "text",
  toDriver: seal,
  fromDriver: unseal,
});
