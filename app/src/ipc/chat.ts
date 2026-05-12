import { invoke } from '@tauri-apps/api/core';
import type { Room } from '../types/ipc';

export const chatListRooms = (): Promise<Room[]> =>
  invoke('chat_list_rooms');

export const chatCreateRoom = (
  name: string,
  relay: string,
  myFp: string,
): Promise<Room> =>
  invoke('chat_create_room', { name, relay, myFp });

export const chatDeleteRoom = (roomId: string): Promise<void> =>
  invoke('chat_delete_room', { roomId });

export const chatAddParticipant = (
  roomId: string,
  participantFp: string,
): Promise<void> =>
  invoke('chat_add_participant', { roomId, participantFp });

export const chatStart = (roomId: string): Promise<void> =>
  invoke('chat_start', { roomId });

export const chatStop = (): Promise<void> =>
  invoke('chat_stop');

export const chatSend = (roomId: string, content: string): Promise<string> =>
  invoke('chat_send', { roomId, content });

export const chatGenerateJoinCode = (roomId: string, myFp: string): Promise<string> =>
  invoke('chat_generate_join_code', { roomId, myFp });

export const chatJoinRoom = (joinCodeStr: string, myFp: string): Promise<Room> =>
  invoke('chat_join_room', { joinCodeStr, myFp });
