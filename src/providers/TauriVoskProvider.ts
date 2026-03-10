import { invoke } from '@tauri-apps/api/tauri'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import type { SpeechProvider, SpeechConfig } from '../types/voice'

export class TauriVoskProvider implements SpeechProvider {
  public name = 'tauri-vosk'
  private streamId: string | null = null
  private resultCallback: ((text: string, isFinal: boolean) => void) | null = null
  private errorCallback: ((error: Error) => void) | null = null
  private endCallback: (() => void) | null = null
  private startCallback: (() => void) | null = null
  private voskUrl: string = 'ws://192.168.150.26:2700'
  private voskApiKey: string = ''
  private silenceTimeout: number = 3000
  private unlistenResult: UnlistenFn | null = null
  private unlistenError: UnlistenFn | null = null
  private unlistenEnd: UnlistenFn | null = null

  setConfig(url: string, apiKey: string): void {
    this.voskUrl = url
    this.voskApiKey = apiKey
  }

  isSupported(): boolean {
    return true // Tauri 总是支持，因为音频由 Rust 处理
  }

  async start(config: SpeechConfig): Promise<void> {
    this.silenceTimeout = config.silenceTimeout ?? 3000
    
    console.log('[TauriVoskProvider] Starting with URL:', this.voskUrl)

    try {
      this.unlistenResult = await listen<{ text: string; is_final: boolean }>('voice_result', (event) => {
        if (this.resultCallback) {
          this.resultCallback(event.payload.text, event.payload.is_final)
        }
      })

      this.unlistenError = await listen<{ message: string }>('voice_error', (event) => {
        if (this.errorCallback) {
          this.errorCallback(new Error(event.payload.message))
        }
      })

      this.unlistenEnd = await listen<string>('voice_end', () => {
        if (this.endCallback) {
          this.endCallback()
        }
        this.cleanup()
      })

      this.streamId = await invoke<string>('start_voice_recognition', {
        voskConfig: {
          url: this.voskUrl,
          api_key: this.voskApiKey || null
        },
        silenceTimeoutMs: this.silenceTimeout
      })

      if (this.startCallback) {
        this.startCallback()
      }
    } catch (e) {
      this.cleanup()
      if (this.errorCallback) {
        this.errorCallback(e as Error)
      }
      throw e
    }
  }

  private cleanup(): void {
    this.unlistenResult?.()
    this.unlistenError?.()
    this.unlistenEnd?.()
    this.unlistenResult = null
    this.unlistenError = null
    this.unlistenEnd = null
  }

  stop(): void {
    if (this.streamId) {
      invoke('stop_voice_recognition', { streamId: this.streamId }).catch(console.error)
      this.streamId = null
    }
    this.cleanup()
  }

  onresult(callback: (text: string, isFinal: boolean) => void): void {
    this.resultCallback = callback
  }

  onerror(callback: (error: Error) => void): void {
    this.errorCallback = callback
  }

  onend(callback: () => void): void {
    this.endCallback = callback
  }

  onstart(callback: () => void): void {
    this.startCallback = callback
  }

  // 静态方法：列出可用麦克风
  static async listMicrophones(): Promise<string[]> {
    return invoke<string[]>('list_microphones')
  }
}