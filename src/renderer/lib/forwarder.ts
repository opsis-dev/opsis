type Send<T> = (message: T) => void;

export type Forwarder<T> = {
  to: (send: Send<T> | null) => void;
  send: (message: T) => boolean;
};

export function createForwarder<T>(): Forwarder<T> {
  let target: Send<T> | null = null;
  return {
    to: (send) => {
      target = send;
    },
    send: (message) => {
      if (!target) return false;
      target(message);
      return true;
    },
  };
}
