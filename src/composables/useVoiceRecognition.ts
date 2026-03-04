import { ref, computed, onUnmounted } from 'vue';
import type { SpeechProvider, SpeechConfig, VoiceInputStatus, ProviderType, VoiceError } from '../types/voice';
import { createProvider } from '../providers';

export interface UseVoiceRecognitionOptions {
  lang?: string;
  continuous?: boolean;
  silenceTimeout?: number;
  provider?: ProviderType;
  voskUrl?: string;
  voskApiKey?: string;
}

export function useVoiceRecognition(options: UseVoiceRecognitionOptions = {}) {
  const status = ref<VoiceInputStatus>('idle');
  const transcript = ref('');
  const interimTranscript = ref('');
  const error = ref<VoiceError | null>(null);

  let provider: SpeechProvider | null = null;

  const isListening = computed(() => status.value === 'listening');
  const isProcessing = computed(() => status.value === 'processing');
  const isIdle = computed(() => status.value === 'idle');
  const hasError = computed(() => status.value === 'error');

  const initProvider = () => {
    try {
      provider = createProvider(options.provider || 'websocket-vosk');
      
      // 配置 Vosk
      if ('setConfig' in provider && options.voskUrl) {
        (provider as any).setConfig(options.voskUrl, options.voskApiKey || '');
      }
      
      provider.onstart(() => {
        status.value = 'listening';
        error.value = null;
      });

      provider.onresult((text, isFinal) => {
        if (isFinal) {
          transcript.value += text;
          interimTranscript.value = '';
          status.value = 'processing';
        } else {
          interimTranscript.value = text;
          status.value = 'listening';
        }
      });

      provider.onerror((err) => {
        error.value = { type: 'unknown', message: err.message };
        status.value = 'error';
      });

      provider.onend(() => {
        if (status.value !== 'error') {
          status.value = 'idle';
        }
      });
    } catch (e) {
      error.value = { 
        type: 'not-supported', 
        message: (e as Error).message 
      };
      status.value = 'error';
    }
  };

  const start = async (text: string = '') => {
    if (!isListening.value && !isProcessing.value) {
      transcript.value = text;
      interimTranscript.value = '';
    }
    
    if (!provider) {
      initProvider();
    }

    if (provider) {
      const config: SpeechConfig = {
        lang: options.lang || 'zh-CN',
        continuous: options.continuous !== false,
        interimResults: true,
        silenceTimeout: options.silenceTimeout || 3000,
      };
      
      try {
        await (provider as any).start(config);
      } catch (e) {
        console.error('Failed to start voice recognition:', e);
      }
    }
  };

  const stop = () => {
    if (provider) {
      provider.stop();
    }
    status.value = 'idle';
  };

  const toggle = async (text: string = '') => {
    if (isListening.value || isProcessing.value) {
      stop();
    } else {
      await start(text);
    }
  };

  const reset = () => {
    transcript.value = '';
    interimTranscript.value = '';
    error.value = null;
    status.value = 'idle';
  };

  onUnmounted(() => {
    if (provider) {
      provider.stop();
    }
  });

  return {
    status,
    transcript,
    interimTranscript,
    error,
    isListening,
    isProcessing,
    isIdle,
    hasError,
    start,
    stop,
    toggle,
    reset,
  };
}
