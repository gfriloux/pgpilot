import { useState, useEffect, useRef, useCallback } from 'react';
import { useChatStore } from '../store/chat';
import { useKeysStore } from '../store/keys';
import {
  chatListRooms,
  chatCreateRoom,
  chatDeleteRoom,
  chatStart,
  chatStop,
  chatSend,
  chatGenerateJoinCode,
  chatJoinRoom,
} from '../ipc/chat';
import type { Room } from '../types/ipc';
import { Button } from '../components/Button';
import { Input } from '../components/Input';
import { Select } from '../components/Select';
import { UssrBanner } from '../components/UssrBanner';
import styles from './Chat.module.css';

// ── Helpers ──────────────────────────────────────────────────────

function formatTime(ts: number): string {
  const d = new Date(ts * 1000);
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

function formatDate(ts: number): string {
  const d = new Date(ts * 1000);
  return d.toLocaleDateString([], { year: 'numeric', month: 'short', day: 'numeric' });
}

function isSameDay(a: number, b: number): boolean {
  const da = new Date(a * 1000);
  const db = new Date(b * 1000);
  return (
    da.getFullYear() === db.getFullYear() &&
    da.getMonth() === db.getMonth() &&
    da.getDate() === db.getDate()
  );
}

function truncateFp(fp: string, len = 8): string {
  return fp.slice(0, len);
}

function truncateRelay(relay: string, max = 36): string {
  if (relay.length <= max) return relay;
  return relay.slice(0, max) + '…';
}

const DEFAULT_RELAY = 'mqtts://broker.hivemq.com:8883';

// ── Connection indicator ──────────────────────────────────────────

interface ConnDotProps {
  connected: boolean;
  reconnecting: boolean;
}

function ConnDot({ connected, reconnecting }: ConnDotProps) {
  if (reconnecting) {
    return (
      <span
        className={styles.reconnectingDot}
        title="Reconnecting…"
        aria-label="Reconnecting"
      />
    );
  }
  if (connected) {
    return (
      <span
        className={styles.connectedDot}
        title="Connected"
        aria-label="Connected"
      />
    );
  }
  return (
    <span
      className={styles.disconnectedDot}
      title="Disconnected"
      aria-label="Disconnected"
    />
  );
}

// ── Create room form ──────────────────────────────────────────────

interface CreateFormProps {
  onCreated: (room: Room) => void;
  onCancel: () => void;
}

function CreateRoomForm({ onCreated, onCancel }: CreateFormProps) {
  const allKeys = useKeysStore((s) => s.keys);
  const secretKeys = allKeys.filter((k) => k.has_secret);

  const [name, setName] = useState('');
  const [relay, setRelay] = useState(DEFAULT_RELAY);
  const [myFp, setMyFp] = useState(secretKeys[0]?.fingerprint ?? '');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const keyOptions = secretKeys.map((k) => ({
    value: k.fingerprint,
    label: `${k.name} <${k.email}>`,
  }));

  function handleSubmit(e: React.FormEvent): void {
    e.preventDefault();
    if (name.trim().length === 0 || myFp.length === 0) return;

    setLoading(true);
    setError(null);
    chatCreateRoom(name.trim(), relay.trim(), myFp)
      .then((room) => {
        onCreated(room);
      })
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => {
        setLoading(false);
      });
  }

  return (
    <form onSubmit={handleSubmit} noValidate className={styles.createForm}>
      <div className={styles.createFormTitle}>New room</div>

      {error !== null && (
        <div style={{ fontSize: '0.75rem', color: 'var(--error)', marginBottom: '4px' }}>
          {error}
        </div>
      )}

      <Input
        label="Room name"
        value={name}
        onChange={(e) => { setName(e.currentTarget.value); }}
        placeholder="Team Alpha"
        disabled={loading}
        autoFocus
      />

      <Input
        label="Relay URL"
        value={relay}
        onChange={(e) => { setRelay(e.currentTarget.value); }}
        placeholder={DEFAULT_RELAY}
        disabled={loading}
      />

      {keyOptions.length > 0 ? (
        <Select
          label="My key"
          options={keyOptions}
          value={myFp}
          onChange={setMyFp}
          disabled={loading}
        />
      ) : (
        <div style={{ fontSize: '0.75rem', color: 'var(--warning)' }}>
          No secret key found. Create a key first.
        </div>
      )}

      <div className={styles.createFormActions}>
        <Button
          variant="ghost"
          size="sm"
          type="button"
          onClick={onCancel}
          disabled={loading}
        >
          Cancel
        </Button>
        <Button
          variant="primary"
          size="sm"
          type="submit"
          loading={loading}
          disabled={keyOptions.length === 0 || name.trim().length === 0}
        >
          Create
        </Button>
      </div>
    </form>
  );
}

// ── Room list item ────────────────────────────────────────────────

interface RoomRowProps {
  room: Room;
  selected: boolean;
  onClick: () => void;
}

function RoomRow({ room, selected, onClick }: RoomRowProps) {
  return (
    <div
      className={`${styles.roomRow}${selected ? ` ${styles.roomRowSelected}` : ''}`}
      onClick={onClick}
      role="option"
      aria-selected={selected}
    >
      <div className={styles.roomRowName}>{room.name}</div>
      <div className={styles.roomRowMeta}>
        <span className={styles.roomRowRelay}>{truncateRelay(room.relay)}</span>
        <span className={styles.roomRowParticipants}>
          {room.participants.length} participant{room.participants.length !== 1 ? 's' : ''}
        </span>
      </div>
    </div>
  );
}

// ── Error Boundary ────────────────────────────────────────────────

import React from 'react';

interface ErrorBoundaryState { error: Error | null }

class ConversationErrorBoundary extends React.Component<
  { children: React.ReactNode },
  ErrorBoundaryState
> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { error: null };
  }
  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }
  render() {
    if (this.state.error !== null) {
      return (
        <div style={{ padding: '24px', color: 'var(--error)', fontFamily: 'var(--font-mono)', fontSize: '0.875rem' }}>
          <strong>Render error:</strong>
          <pre style={{ marginTop: '8px', whiteSpace: 'pre-wrap' }}>{this.state.error.message}</pre>
        </div>
      );
    }
    return this.props.children;
  }
}

// ── Conversation panel ────────────────────────────────────────────

interface ConversationProps {
  room: Room;
  onClose: () => void;
}

function Conversation({ room, onClose }: ConversationProps) {
  const connected = useChatStore((s) => s.connected);
  const reconnecting = useChatStore((s) => s.reconnecting);
  const rawMessages = useChatStore((s) => s.messages[room.id]);
  const messages = rawMessages ?? [];
  const allKeys = useKeysStore((s) => s.keys);

  const [draft, setDraft] = useState('');
  const [sending, setSending] = useState(false);
  const [startError, setStartError] = useState<string | null>(null);
  const [expandedMsgId, setExpandedMsgId] = useState<string | null>(null);
  const bottomRef = useRef<HTMLDivElement | null>(null);

  function senderName(fp: string): string {
    return allKeys.find((k) => k.fingerprint === fp)?.name ?? truncateFp(fp);
  }

  function headerClass(fp: string): string {
    const trust = allKeys.find((k) => k.fingerprint === fp)?.trust;
    if (trust === 'full' || trust === 'ultimate') return `${styles.messageBubbleHeader} ${styles.messageBubbleHeaderTrusted}`;
    if (trust === 'marginal') return `${styles.messageBubbleHeader} ${styles.messageBubbleHeaderMarginal}`;
    if (trust === 'undefined') return `${styles.messageBubbleHeader} ${styles.messageBubbleHeaderUntrusted}`;
    return `${styles.messageBubbleHeader} ${styles.messageBubbleHeaderUnknown}`;
  }

  function lockTitle(fp: string): string {
    const trust = allKeys.find((k) => k.fingerprint === fp)?.trust;
    if (trust === 'full' || trust === 'ultimate') return 'Signature valid — key trusted';
    if (trust === 'marginal') return 'Signature valid — marginal trust';
    if (trust === 'undefined') return 'Signature valid — key not trusted in GPG';
    return 'Signature valid — key unknown locally';
  }

  function handleCopyInvite(): void {
    chatGenerateJoinCode(room.id, room.my_fp)
      .then((code) => navigator.clipboard.writeText(code))
      // eslint-disable-next-line no-console
      .catch((err: unknown) => console.error('Copy invite failed', err));
  }

  // Start MQTT on mount, stop on unmount.
  // No StrictMode guard — chatStop on cleanup is harmless (idempotent).
  useEffect(() => {
    let cancelled = false;
    chatStart(room.id).catch((err: unknown) => {
      if (!cancelled) {
        const msg = err instanceof Error ? err.message : String(err);
        setStartError(msg);
      }
    });
    return () => {
      cancelled = true;
      chatStop().catch(() => undefined);
    };
  }, [room.id]);

  // Scroll to bottom when new messages arrive
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages.length]);

  const handleSend = useCallback(() => {
    const content = draft.trim();
    if (content.length === 0 || sending) return;
    setSending(true);
    chatSend(room.id, content)
      .then(() => {
        setDraft('');
      })
      .catch(() => { /* status already shown via events */ })
      .finally(() => {
        setSending(false);
      });
  }, [draft, room.id, sending]);

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>): void {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }

  if (startError !== null) {
    return (
      <div style={{ padding: '24px', color: 'var(--error)', fontFamily: 'var(--font-mono)', fontSize: '0.875rem' }}>
        <strong>Chat connection failed:</strong>
        <pre style={{ marginTop: '8px', whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>{startError}</pre>
      </div>
    );
  }

  return (
    <>
      {/* Header */}
      <div className={styles.convHeader}>
        <div className={styles.convHeaderLeft}>
          <span className={styles.convHeaderName}>{room.name}</span>
          <div className={styles.convHeaderStatus}>
            <ConnDot connected={connected} reconnecting={reconnecting} />
            <span>
              {reconnecting ? 'Reconnecting…' : connected ? 'Connected' : 'Disconnected'}
            </span>
          </div>
        </div>
        <div style={{ display: 'flex', gap: '6px' }}>
          <Button variant="ghost" size="sm" onClick={() => { void handleCopyInvite(); }}>
            Copy invite
          </Button>
          <Button variant="ghost" size="sm" onClick={onClose} aria-label="Close conversation">
            &#x2715;
          </Button>
        </div>
      </div>

      {/* Connection banners */}
      {reconnecting && (
        <div className={styles.bannerReconnecting} role="alert">
          Reconnexion en cours…
        </div>
      )}
      {!reconnecting && !connected && (
        <div className={styles.bannerDisconnected} role="alert">
          Deconnecte
        </div>
      )}

      {/* Messages */}
      <div className={styles.messageList} aria-live="polite" aria-label="Messages">
        {messages.length === 0 && (
          <div style={{ textAlign: 'center', color: 'var(--text-muted)', fontSize: '0.875rem', margin: 'auto' }}>
            No messages yet. Send the first one!
          </div>
        )}

        {messages.map((msg, idx) => {
          const isOwn = msg.sender_fp === room.my_fp;
          const prevMsg: typeof messages[number] | null =
            idx > 0 ? (messages[idx - 1] ?? null) : null;
          const showDateSep =
            prevMsg === null || !isSameDay(prevMsg.ts, msg.ts);

          return (
            <div key={msg.msg_id}>
              {showDateSep && (
                <div className={styles.dateSeparator}>
                  {formatDate(msg.ts)}
                </div>
              )}
              <div
                className={
                  isOwn
                    ? `${styles.messageBubbleWrapper} ${styles.messageBubbleWrapperMine}`
                    : `${styles.messageBubbleWrapper} ${styles.messageBubbleWrapperOther}`
                }
              >
                <div
                  className={
                    isOwn
                      ? `${styles.messageBubble} ${styles.messageBubbleMine}`
                      : styles.messageBubble
                  }
                  onClick={() => setExpandedMsgId(
                    expandedMsgId === msg.msg_id ? null : msg.msg_id
                  )}
                  role="button"
                  tabIndex={0}
                  aria-expanded={expandedMsgId === msg.msg_id}
                >
                  {/* Header: lock + name · time */}
                  <div className={headerClass(msg.sender_fp)}>
                    <span className={styles.messageLock} title={lockTitle(msg.sender_fp)}>&#128274;</span>
                    <span className={styles.messageSender}>{senderName(msg.sender_fp)}</span>
                    <span className={styles.messageDot}>·</span>
                    <span className={styles.messageTime}>{formatTime(msg.ts)}</span>
                  </div>
                  {/* Body */}
                  <div className={styles.messageBubbleContent}>
                    {msg.content}
                  </div>
                </div>
                {expandedMsgId === msg.msg_id && (
                  <div className={styles.messageDetail}>
                    <span>{msg.sender_fp}</span>
                    <span>{new Date(msg.ts * 1000).toLocaleString()}</span>
                  </div>
                )}
              </div>
            </div>
          );
        })}
        <div ref={bottomRef} />
      </div>

      {/* Input footer */}
      <div className={styles.messageInput}>
        <input
          className={styles.messageInputField}
          type="text"
          placeholder="Type a message… (Enter to send)"
          value={draft}
          onChange={(e) => { setDraft(e.currentTarget.value); }}
          onKeyDown={handleKeyDown}
          disabled={sending}
          aria-label="Message"
          maxLength={4096}
        />
        <Button
          variant="primary"
          size="sm"
          onClick={handleSend}
          disabled={draft.trim().length === 0 || sending}
          loading={sending}
        >
          Send
        </Button>
      </div>
    </>
  );
}

// ── JoinRoomForm ──────────────────────────────────────────────────

interface JoinRoomFormProps {
  onJoined: (room: Room) => void;
  onCancel: () => void;
}

function JoinRoomForm({ onJoined, onCancel }: JoinRoomFormProps) {
  const keys = useKeysStore((s) => s.keys).filter((k) => k.has_secret);
  const [code, setCode] = useState('');
  const [myFp, setMyFp] = useState(keys[0]?.fingerprint ?? '');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function handleJoin(): void {
    if (!code.trim() || !myFp) return;
    setLoading(true);
    setError(null);
    chatJoinRoom(code.trim(), myFp)
      .then((room) => { onJoined(room); })
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => { setLoading(false); });
  }

  const keyOptions = keys.map((k) => ({
    value: k.fingerprint,
    label: `${k.name} <${k.email}>`,
  }));

  return (
    <div className={styles.createForm}>
      <p className={styles.createFormTitle}>Join via invitation</p>
      <Input
        label="Invitation code"
        value={code}
        onChange={(e) => { setCode(e.currentTarget.value); }}
        hint="Paste the pgpilot:join:... code"
      />
      {keyOptions.length > 0 && (
        <Select
          label="My key"
          options={keyOptions}
          value={myFp}
          onChange={setMyFp}
        />
      )}
      {error !== null && (
        <p style={{ fontSize: '0.75rem', color: 'var(--error)' }}>{error}</p>
      )}
      <div className={styles.createFormActions}>
        <Button variant="ghost" size="sm" onClick={onCancel}>Cancel</Button>
        <Button
          variant="primary"
          size="sm"
          loading={loading}
          disabled={!code.trim() || !myFp}
          onClick={handleJoin}
        >
          Join
        </Button>
      </div>
    </div>
  );
}

// ── Main Chat page ────────────────────────────────────────────────

export default function Chat() {
  const rooms = useChatStore((s) => s.rooms);
  const selectedRoomId = useChatStore((s) => s.selectedRoomId);
  const setRooms = useChatStore((s) => s.setRooms);
  const selectRoom = useChatStore((s) => s.selectRoom);
  const addRoom = useChatStore((s) => s.addRoom);
  const deleteRoomFromStore = useChatStore((s) => s.deleteRoom);
  const connected = useChatStore((s) => s.connected);
  const reconnecting = useChatStore((s) => s.reconnecting);

  const [showCreate, setShowCreate] = useState(false);
  const [showJoin, setShowJoin] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);

  // Load rooms on mount
  useEffect(() => {
    chatListRooms()
      .then(setRooms)
      .catch((err: unknown) => {
        setLoadError(err instanceof Error ? err.message : String(err));
      });
  }, [setRooms]);

  const selectedRoom =
    selectedRoomId !== null
      ? rooms.find((r) => r.id === selectedRoomId) ?? null
      : null;

  function handleRoomCreated(room: Room): void {
    addRoom(room);
    setShowCreate(false);
    selectRoom(room.id);
  }

  function handleDeleteRoom(roomId: string): void {
    chatDeleteRoom(roomId).catch(() => { /* ignore */ });
    deleteRoomFromStore(roomId);
  }

  function handleClose(): void {
    chatStop().catch(() => { /* ignore */ });
    selectRoom(null);
  }

  return (
    <div className={styles.page}>
      {/* ── List panel ─────────────────────────────────────── */}
      <div className={styles.listPanel}>
        <div className={styles.listHeader}>
          <span className={styles.listTitle}>Chat</span>
          <div className={styles.listHeaderRight}>
            {selectedRoomId !== null && (
              <ConnDot connected={connected} reconnecting={reconnecting} />
            )}
            <Button
              variant="ghost"
              size="sm"
              onClick={() => { setShowJoin((v) => !v); setShowCreate(false); }}
              aria-label="Join room via invitation"
            >
              Join
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => { setShowCreate((v) => !v); setShowJoin(false); }}
              aria-label="Create new room"
            >
              {showCreate ? '−' : '+'}
            </Button>
          </div>
        </div>

        {/* Inline create form */}
        {showCreate && (
          <CreateRoomForm
            onCreated={handleRoomCreated}
            onCancel={() => { setShowCreate(false); }}
          />
        )}

        {/* Inline join form */}
        {showJoin && (
          <JoinRoomForm
            onJoined={(room) => { addRoom(room); setShowJoin(false); selectRoom(room.id); }}
            onCancel={() => { setShowJoin(false); }}
          />
        )}

        <UssrBanner n={26} />

        <div
          className={styles.roomList}
          role="listbox"
          aria-label="Chat rooms"
          aria-orientation="vertical"
        >
          {loadError !== null && (
            <p className={styles.roomListEmpty} style={{ color: 'var(--error)' }}>
              Error loading rooms: {loadError}
            </p>
          )}

          {loadError === null && rooms.length === 0 && !showCreate && (
            <p className={styles.roomListEmpty}>
              No rooms yet. Create one with the + button.
            </p>
          )}

          {rooms.map((room) => (
            <RoomRow
              key={room.id}
              room={room}
              selected={room.id === selectedRoomId}
              onClick={() => { selectRoom(room.id); }}
            />
          ))}
        </div>

        {/* Delete selected room */}
        {selectedRoom !== null && (
          <div
            style={{
              padding: '8px 12px',
              borderTop: '1px solid var(--border)',
              flexShrink: 0,
            }}
          >
            <Button
              variant="destructive"
              size="sm"
              onClick={() => { handleDeleteRoom(selectedRoom.id); }}
              style={{ width: '100%' }}
            >
              Delete room
            </Button>
          </div>
        )}
      </div>

      {/* ── Detail / conversation panel ──────────────────── */}
      <div className={styles.detailPanel}>
        {selectedRoom !== null ? (
          <ConversationErrorBoundary key={selectedRoom.id}>
            <Conversation
              room={selectedRoom}
              onClose={handleClose}
            />
          </ConversationErrorBoundary>
        ) : (
          <div className={styles.detailPlaceholder}>
            &#8592; Select a room
          </div>
        )}
      </div>
    </div>
  );
}
