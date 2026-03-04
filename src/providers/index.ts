import type { SpeechProvider, ProviderType } from '../types/voice';
import { WebSocketVoskProvider } from './WebSocketVoskProvider';

export function createProvider(type: ProviderType): SpeechProvider {
  switch (type) {
    case 'websocket-vosk':
    default:
      return new WebSocketVoskProvider();
  }
}

export { WebSocketVoskProvider } from './WebSocketVoskProvider';
