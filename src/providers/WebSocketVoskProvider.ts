import type { SpeechProvider, SpeechConfig } from '../types/voice';

export class WebSocketVoskProvider implements SpeechProvider {
  public name = 'websocket-vosk';
  private ws: WebSocket | null = null;
  private resultCallback: ((text: string, isFinal: boolean) => void) | null = null;
  private errorCallback: ((error: Error) => void) | null = null;
  private endCallback: (() => void) | null = null;
  private startCallback: (() => void) | null = null;
  private silenceTimer: number | null = null;
  private silenceTimeout: number = 3000;
  private mediaStream: MediaStream | null = null;
  private audioContext: AudioContext | null = null;
  private processor: ScriptProcessorNode | null = null;
  private wsUrl: string = 'ws://localhost:5000';
  private apiKey: string = '';

  setConfig(url: string, apiKey: string): void {
    this.wsUrl = url;
    this.apiKey = apiKey;
  }

  isSupported(): boolean {
    return typeof window !== 'undefined' && 
           'WebSocket' in window && 
           'MediaRecorder' in window;
  }

  private connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        const url = this.apiKey 
          ? `${this.wsUrl}?api_key=${this.apiKey}`
          : this.wsUrl;
        
        this.ws = new WebSocket(url);
        
        this.ws.onopen = () => {
          console.log('[WebSocket Vosk] Connected');
          resolve();
        };
        
        this.ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            if (data.result) {
              const text = data.result.map((r: any) => r.word).join(' ');
              if (this.resultCallback) {
                this.resultCallback(text, true);
              }
              this.resetSilenceTimer();
            } else if (data.partial) {
              if (this.resultCallback) {
                this.resultCallback(data.partial, false);
              }
            }
          } catch (e) {
            console.error('[WebSocket Vosk] Parse error:', e);
          }
        };
        
        this.ws.onerror = (error) => {
          console.error('[WebSocket Vosk] Error:', error);
          if (this.errorCallback) {
            this.errorCallback(new Error('WebSocket connection error'));
          }
          reject(error);
        };
        
        this.ws.onclose = () => {
          console.log('[WebSocket Vosk] Closed');
          if (this.endCallback) {
            this.endCallback();
          }
        };
      } catch (e) {
        reject(e);
      }
    });
  }

  private resetSilenceTimer(): void {
    if (this.silenceTimer) {
      clearTimeout(this.silenceTimer);
    }
    if (this.silenceTimeout > 0) {
      this.silenceTimer = window.setTimeout(() => {
        this.stop();
      }, this.silenceTimeout);
    }
  }

  async start(config: SpeechConfig): Promise<void> {
    this.silenceTimeout = config.silenceTimeout ?? 3000;
    
    try {
      // 连接 WebSocket
      await this.connect();
      
      // 获取麦克风权限
      this.mediaStream = await navigator.mediaDevices.getUserMedia({ audio: true });
      
      // 创建音频处理
      this.audioContext = new AudioContext();
      const source = this.audioContext.createMediaStreamSource(this.mediaStream);
      this.processor = this.audioContext.createScriptProcessor(4096, 1, 1);
      
      this.processor.onaudioprocess = (event) => {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
          const inputData = event.inputBuffer.getChannelData(0);
          // 转换为 16 位 PCM
          const pcmData = this.audioBufferTo16BitPCM(inputData);
          this.ws.send(pcmData);
        }
      };
      
      source.connect(this.processor);
      this.processor.connect(this.audioContext.destination);
      
      if (this.startCallback) {
        this.startCallback();
      }
      
      this.resetSilenceTimer();
    } catch (e) {
      if (this.errorCallback) {
        this.errorCallback(e as Error);
      }
      throw e;
    }
  }

  private audioBufferTo16BitPCM(float32Array: Float32Array): ArrayBuffer {
    const buffer = new ArrayBuffer(float32Array.length * 2);
    const view = new DataView(buffer);
    for (let i = 0; i < float32Array.length; i++) {
      const s = Math.max(-1, Math.min(1, float32Array[i]));
      view.setInt16(i * 2, s < 0 ? s * 0x8000 : s * 0x7FFF, true);
    }
    return buffer;
  }

  stop(): void {
    if (this.silenceTimer) {
      clearTimeout(this.silenceTimer);
      this.silenceTimer = null;
    }
    
    if (this.processor) {
      this.processor.disconnect();
      this.processor = null;
    }
    
    if (this.audioContext) {
      this.audioContext.close();
      this.audioContext = null;
    }
    
    if (this.mediaStream) {
      this.mediaStream.getTracks().forEach(track => track.stop());
      this.mediaStream = null;
    }
    
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  onresult(callback: (text: string, isFinal: boolean) => void): void {
    this.resultCallback = callback;
  }

  onerror(callback: (error: Error) => void): void {
    this.errorCallback = callback;
  }

  onend(callback: () => void): void {
    this.endCallback = callback;
  }

  onstart(callback: () => void): void {
    this.startCallback = callback;
  }
}
