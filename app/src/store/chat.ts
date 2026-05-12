import { create } from 'zustand';
import type { Room, ChatMessage } from '../types/ipc';

const MAX_MESSAGES_PER_ROOM = 500;

interface ChatStore {
  rooms: Room[];
  selectedRoomId: string | null;
  /** Messages in RAM only — max 500 per room (FIFO) */
  messages: Record<string, ChatMessage[]>;
  connected: boolean;
  reconnecting: boolean;

  setRooms: (rooms: Room[]) => void;
  selectRoom: (id: string | null) => void;
  pushMessage: (msg: ChatMessage) => void;
  setConnected: (v: boolean) => void;
  setReconnecting: (v: boolean) => void;
  deleteRoom: (id: string) => void;
  addRoom: (room: Room) => void;
}

export const useChatStore = create<ChatStore>()((set) => ({
  rooms: [],
  selectedRoomId: null,
  messages: {},
  connected: false,
  reconnecting: false,

  setRooms: (rooms) => set({ rooms }),

  selectRoom: (id) => set({ selectedRoomId: id }),

  pushMessage: (msg) =>
    set((state) => {
      const existing = state.messages[msg.room_id] ?? [];
      const updated =
        existing.length >= MAX_MESSAGES_PER_ROOM
          ? [...existing.slice(1), msg]
          : [...existing, msg];
      return {
        messages: {
          ...state.messages,
          [msg.room_id]: updated,
        },
      };
    }),

  setConnected: (connected) => set({ connected }),

  setReconnecting: (reconnecting) => set({ reconnecting }),

  deleteRoom: (id) =>
    set((state) => {
      const rooms = state.rooms.filter((r) => r.id !== id);
      const messages = { ...state.messages };
      delete messages[id];
      const selectedRoomId =
        state.selectedRoomId === id ? null : state.selectedRoomId;
      return { rooms, messages, selectedRoomId };
    }),

  addRoom: (room) =>
    set((state) => ({
      rooms: [...state.rooms, room],
    })),
}));
