# Voice Wake Keyword Spotting Design

## Overview

The current voice wake functionality in the shine-helper application uses only energy-based Voice Activity Detection (VAD) triggered on simple energy threshold (0.02). This means it responds to any loud sound rather than recognizing specific keywords like "小 Shine". This design addresses the requirements by implementing actual keyword spotting functionality.

## Purpose

Enable the application to respond only to specifically configured wake words instead of any loud audio sounds while maintaining existing features.

## Problem Statement

- Current system triggers on any audio that exceeds the energy threshold of 0.02 (Voice Activity Detection style)
- Does not actually recognize specific phrases like "小 Shine"
- Responds to all loud noises, not just intended wake word
- Poor user experience due to false positives

## Proposed Approaches

### 1. Pure ASR-Based Approach (Recommended)
- Use existing VOSK transcription continuously in listening mode
- Compare transcribed text against the configured wake word
- Pros: Works well for Chinese language recognition, leverages existing infrastructure
- Cons: Higher resource consumption, potential latency

### 2. Audio Feature Matching
- Extract audio features (MFCC, mel-spectrogram) for specific wake word
- Use ML model to match these features only when energy threshold is met
- Pros: Lower resource usage when idle, very fast response time
- Cons: Requires ML model training for each specific language/model

### 3. Hybrid VAD+ASR Approach (Recommended)
- Use current energy-based VAD as initial trigger
- Feed audio to lightweight ASR when activity detected
- Match against configured wake word
- Pros: Balances performance and accuracy, leverages existing code
- Cons: Slight processing overhead after initial VAD trigger

## Recommended Solution

Hybrid VAD+ASR Approach is recommended because:
- Leverages existing infrastructure and codebase
- Maintains current responsiveness
- Uses configuration settings appropriately
- Relatively simple compared to complex ML approaches

## Architecture

```
Audio Input
     ↓
Energy Threshold Detection (current: > 0.02) 
     ↓ (if exceeded)
Audio Buffer for ASR Processing
     ↓
VOSK ASR Transcription Service
     ↓
Keyword String Matching Against Configured Value
     ↓ (if match)
Activate Wake Sequence (Window Focus + State Transition)
```

## Components

1. **Wake Word Configuration** - Already exists in AppConfig::voice_wake.wake_word
2. **Audio Buffering** - Temporary storage of audio segments during wake detection
3. **ASR Processing Module** - Lightweight VOSK connection to transcribe potential wake word
4. **String Matcher** - Compare transcribed text against configured wake word
5. **State Management** - Maintain existing state transitions

## Data Flow

1. Continuous energy monitoring using current algorithm
2. When energy > threshold, capture audio segment in buffer (e.g. last ~2s of audio)
3. Perform fast ASR on the buffer using VOSK
4. Compare ASR result against configured wake_word
5. If match, trigger wake response

## Error Handling

- Handle ASR processing failures by falling back to current behavior
- Implement timeout for ASR processing to prevent hanging
- Log ASR confidence scores for debugging

## Testing Strategy

- Unit tests for keyword matching functionality
- Integration tests with audio processing pipeline
- Manual testing with actual wake word vs other phrases