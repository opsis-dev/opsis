import { isRecord } from "@opsis/video/common/guards";
import type { RelayTarget } from "@/generated/bindings";

type Listener = (event: { data: unknown }) => void;

export type RelayChannel = {
  postMessage: (message: unknown) => void;
  addEventListener: (type: "message", listener: Listener) => void;
  close: () => void;
};

export function relayChannel(name: string, to: RelayTarget = "others"): RelayChannel {
  const listeners = new Set<Listener>();
  const off = window.opsis.onRelayed(({ message }) => {
    if (!isRecord(message) || message.channel !== name) return;
    for (const listener of listeners) listener({ data: message.message });
  });
  return {
    postMessage: (message) => void window.opsis.relay(to, { channel: name, message }),
    addEventListener: (_type, listener) => void listeners.add(listener),
    close: () => {
      off();
      listeners.clear();
    },
  };
}
