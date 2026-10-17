const read = (storage: () => Storage, key: string): string | null => {
  try {
    return storage().getItem(key);
  } catch {
    return null;
  }
};

const write = (storage: () => Storage, key: string, value: string): void => {
  try {
    storage().setItem(key, value);
  } catch {
    return;
  }
};

const local = () => window.localStorage;
const session = () => window.sessionStorage;

export const readStored = (key: string): string | null => read(local, key);

export const writeStored = (key: string, value: string): void => write(local, key, value);

export const readSession = (key: string): string | null => read(session, key);

export const writeSession = (key: string, value: string): void => write(session, key, value);
