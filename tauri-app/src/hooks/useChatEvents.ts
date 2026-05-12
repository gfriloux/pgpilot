import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useChatStore } from '../store/chat';
import type { ChatMessage } from '../types/ipc';

export function useChatEvents(): void {
  const setConnected = useChatStore((s) => s.setConnected);
  const setReconnecting = useChatStore((s) => s.setReconnecting);
  const pushMessage = useChatStore((s) => s.pushMessage);

  useEffect(() => {
    // Store the Promises themselves so cleanup can always resolve + call unlisten,
    // even in React StrictMode where cleanup may run before the Promises resolve.
    const promises: Promise<() => void>[] = [
      listen<null>('chat:connected', () => {
        setConnected(true);
        setReconnecting(false);
      }),
      listen<{ reason: string }>('chat:disconnected', () => {
        setConnected(false);
      }),
      listen<{ attempt: number }>('chat:reconnecting', () => {
        setReconnecting(true);
      }),
      listen<ChatMessage>('chat:message', (event) => {
        pushMessage(event.payload);
      }),
    ];

    return () => {
      promises.forEach((p) => void p.then((u) => u()).catch(() => undefined));
    };
  }, [setConnected, setReconnecting, pushMessage]);
}
