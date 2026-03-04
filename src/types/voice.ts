export interface SpeechConfig {
  lang: string;
  continuous: boolean;
  interimResults: boolean;
  silenceTimeout?: number;
}

export type VoiceInputStatus = 'idle' | 'listening' | 'processing' | 'error';

export type ProviderType = 'websocket-vosk';

export interface VoiceInputProps {
  lang?: string;
  continuous?: boolean;
  silenceTimeout?: number;
  placeholder?: string;
  modelValue?: string;
}

export interface VoiceInputEvents {
  'update:modelValue': (value: string) => void;
  'start': () => void;
  'end': () => void;
  'error': (error: Error) => void;
  'result': (text: string, isFinal: boolean) => void;
}

export interface SpeechProvider {
  name: string;
  isSupported(): boolean;
  start(config: SpeechConfig): void;
  stop(): void;
  onresult(callback: (text: string, isFinal: boolean) => void): void;
  onerror(callback: (error: Error) => void): void;
  onend(callback: () => void): void;
  onstart(callback: () => void): void;
}

export interface VoiceError {
  type: 'not-supported' | 'no-microphone' | 'permission-denied' | 'network-error' | 'no-speech' | 'service-not-allowed' | 'unknown';
  message: string;
}
