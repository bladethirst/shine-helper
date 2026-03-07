import type { SpeechProvider, ProviderType } from '../types/voice';
import { WebSocketVoskProvider } from './WebSocketVoskProvider';
import { TauriVoskProvider } from './TauriVoskProvider';

export function createProvider(type: ProviderType): SpeechProvider {
  switch (type) {
    case 'tauri-vosk':
      return new TauriVoskProvider();
    case 'websocket-vosk':
    default:
      return new WebSocketVoskProvider();
  }
}

export { WebSocketVoskProvider } from './WebSocketVoskProvider';
export { TauriVoskProvider } from './TauriVoskProvider';
