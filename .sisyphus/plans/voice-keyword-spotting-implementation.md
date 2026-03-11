# Voice Wake Keyword Spotting Implementation Plan

## TL;DR

> **Quick Summary**: Implement true keyword spotting in the voice wake functionality to recognize "小 Shine" and other configured wake words instead of just using energy threshold detection
> 
> **Deliverables**: Enhanced voice wake with keyword matching, backward compatibility, improved configuration
> 
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - N waves
> **Critical Path**: Task 1 → Task 2 → Task 3 → Task 4

---

## Context

### Original Request
User wants to implement actual keyword spotting functionality in the voice wake system to properly recognize the configured wake word ("小 Shine") instead of just responding to any sound that exceeds the energy threshold.

### Design Decisions
- **Approach**: Hybrid VAD+ASR - Use energy threshold for initial activity detection but then perform keyword matching
- **Technology**: Leverage existing VOSK ASR infrastructure for keyword verification
- **Integration**: Maintain backward compatibility with existing state machine
- **Performance**: Include caching/loading optimizations to avoid re-instantiating VOSK constantly

### Research Findings
- Application currently uses energy-based VAD only - any loud audio >0.02 triggers wake
- VOSK integration already exists in codebase for transcription
- Configuration already allows configurable wake word (though unused)
- Window focus functionality already implemented correctly

---

## Work Objectives

### Core Objective
Replace the simple energy threshold wake detection with actual keyword spotting that verifies audio content matches the configured wake word before triggering wake events.

### Concrete Deliverables
- Modified voice wake processing that performs keyword verification after initial energy detection
- Keyword matching logic integrated into existing wake process
- Configurable wake word that is actually used for keyword detection
- Logging improvements to indicate keyword matching successes/failures

### Definition of Done
- [ ] Audio energy exceeding threshold triggers keyword verification (not immediate wake)
- [ ] Only configured wake word triggers actual wake event
- [ ] Wake verification failure logs message indicating word not recognized
- [ ] Window focus and UI events only occur when actual wake word recognized
- [ ] Configuration is utilized for actual keyword matching
- [ ] Backward compatibility maintained with existing features

### Must Have
- True keyword spot functionality (not just audio energy)
- Use configured wake word for matching
- Maintain existing wake functionality and window focus behavior
- Preserve auto-start functionality for configuration check on launch

### Must NOT Have (Guardrails)
- Remove energy threshold completely (keep as initial detection)
- Sacrifice accuracy for performance
- Break existing VOSK transcription flow
- Lose current window focus functionality

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (existing cargo test infrastructure in workspace)
- **Automated tests**: TDD / none (minimal - will primarily be manual testing since this is audio/real-time related)
- **Framework**: Test using mock configuration scenarios and simulated audio events
- **If TDD**: Each new verification component includes basic positive/negative tests for keyword matching

### QA Policy
Every task MUST include agent-executed QA scenarios (see TODO template below).
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Component testing**: Rust unit tests for new keyword matching functions
- **Integration testing**: Manual verification of wake word recognition vs other phrases
- **Configuration testing**: Different wake words work correctly

---

## Execution Strategy

### Parallel Execution Waves

> Maximize throughput by grouping independent tasks into parallel waves.
> Each wave completes before the next begins.
> Target: 5-8 tasks per wave. < 3 per wave = under-splitting.

```
Wave 1 (Start Immediately — infrastructure setup):
├── Task 1: Update wake word detection logic with keyword matching [deep]
├── Task 2: Modify configuration handling to use wake word field [deep]
└── Task 3: Add keyword spot logging and diagnostics [quick]

Wave 2 (After Wave 1 — core audio integration):
├── Task 4: Integrate ASR processing with keyword verification [deep]
├── Task 5: Update UI emission triggers to require keyword match [deep]
└── Task 6: Add audio buffering for keyword verification capture [unspecified-high]

Wave 3 (After Waves 1&2 — refinement):
├── Task 7: Create keyword matching utility functions [visual-engineering]
├── Task 8: Update error handling for ASR processing [deep]
├── Task 9: Test wake word variations and configuration changes [quick]

Wave 4 (After ALL tasks — verification and cleanup):
├── Task 10: Integration test with manual verification [deep]
├── Task 11: Performance verification [visual-engineering]
└── Task 12: Documentation update [writing]

Wave FINAL (After ALL tasks — independent review, 4 parallel):
├── Task F1: Design compliance audit [oracle]
├── Task F2: Performance and resource audit [unspecified-high]
├── Task F3: Keyword spot accuracy verification [unspecified-high]
├── Task F4: Configuration consistency check [deep]

Critical Path: Task 1 → Task 4 → Task 5 → Task 10
Parallel Speedup: ~65% faster than sequential
Max Concurrent: 6 (Waves 1&2)
```

### Dependency Matrix (abbreviated — show ALL tasks in your generated plan)

- **1**: — — 2,7,1
- **2**: 1 — 4,5,8,2
- **3**: 1 — 4,5,3
- **4**: 1,2,3 — 5,6,7,9,4
- **5**: 4 — 7,10,5
- **6**: 4 — 7,6
- **7**: 4,6 — 8,9,7
- **8**: 7 — 10,8
- **9**: 7 — 10,9
- **10**: 5,8,9 — 12,2
- **11**: — — 12,1
- **12**: 10,11 — F1-F4,F4,4

> This is abbreviated for reference. YOUR generated plan must include the FULL matrix for ALL tasks.

### Agent Dispatch Summary

- **1**: **4** — T1 → `deep`, T2 → `deep`, T3 → `quick`
- **2**: **3** — T4 → `deep`, T5 → `deep`, T6 → `unspecified-high`
- **3**: **3** — T7 → `visual-engineering`, T8 → `deep`, T9 → `quick`
- **4**: **2** — T10 → `deep`, T11 → `visual-engineering`, T12 → `writing`
- **FINAL**: **4** — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

> Implementation + Test = ONE Task. Never separate.
> EVERY task MUST have: Recommended Agent Profile + Parallelization info + QA Scenarios.
> **A task WITHOUT QA Scenarios is INCOMPLETE. No exceptions.**

- [ ] 1. Update wake word detection logic with keyword matching

  **What to do**:
  - Modify existing voice wake loop to perform keyword verification after energy threshold crossed
  - Change logic from `if energy > threshold { trigger_wake() }` to `if energy > threshold { verify_keyword_match() }`
  - Only trigger wake if ASR result matches configured wake word from config
  - Implement buffer for capturing audio segment prior to trigger for keyword verification

  **Must NOT do**:
  - Remove energy detection completely (still use for initial activity detection)
  - Skip ASR for keyword verification
  - Process all audio continuously (only when potential wake detected)

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `deep`
    - Reason: Requires deep understanding of Rust async audio processing and ASR integration
  - **Skills**: []
    - ``: [Why needed - domain overlap explanation]

  **Parallelization**:
  - **Can Run In Parallel**: NO - Critical path item
  - **Parallel Group**: Wave N () | Sequential  
  - **Blocks**: [Tasks that depend on this task completing] | None (can start immediately)
  - **Blocked By**: [] | None (can start immediately)

  **References** (CRITICAL - Be Exhaustive):

  > The executor has NO context from your interview. References are their ONLY guide.
  > Each reference must answer: "What should I look at and WHY?"

  **Pattern References** (existing code to follow):
  - `src-tauri/src/commands/voice_wake.rs:220-290` - Current voice loop energy threshold checking logic

  **API/Type References** (contracts to implement against):
  - `src-tauri/src/config.rs:37-44` - VoiceWakeConfig structure with wake_word field

  **Test References** (testing patterns to follow):
  - `src-tauri/src/commands/voice_wake.rs:70-137` - test_voice_wake_detection shows audio processing patterns

  **External References** (libraries and frameworks):
  - https://github.com/robiot/vosk-rs - Vosk-rs library for ASR
  - https://github.com/serde-rs/json - Serde_json for ASR responses parsing

  **WHY Each Reference Matters** (explain the relevance):
  - Allowing understanding of current energy comparison implementation to know where to insert keyword matching
  - Looking at configuration structure to understand how wake_word is currently stored
  - Studying test functions to understand audio processing patterns used elsewhere in app

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.
  > Every criterion MUST be vervable by running a command or using a tool.

  **If TDD (tests enabled):**
  - [ ] Update `run_wake_loop` function in voice_wake.rs to implement keyword verification
  - [ ] bun test src-tauri/src/commands/voice_wake_test.rs → PASS (if such file exists)

  **QA Scenarios (MANDATORY):**

  > **This is NOT optional. A task without QA scenarios WILL BE REJECTED.**
  >
  > Write scenario tests that verify the ACTUAL BEHAVIOR of what you built.
  > Minimum: 1 happy path + 1 failure/edge case per task.
  > Each scenario = exact tool + exact steps + exact assertions + evidence path.
  >
  > **The executing agent MUST run these scenarios after implementation.**
  > **The orchestrator WILL verify evidence files exist before marking task complete.**

  ```
  Scenario: [Keyword matching works when audio contains configured wake word]
    Tool: Interactive rust build/test 
    Preconditions: Application is running, VOSK service available at configured URL
    Steps:
      1. Simulate audio input containing actual configured wake word ("小 Shine")
      2. Verify energy threshold exceeded in current flow to trigger keyword matching
      3. Confirm ASR processes input and recognizes configured wake word
      4. Verify window focus occurs when keyword verified
      5. Check that 'voice-waked' event is emitted
    Expected Result: Application window comes to frontend and emits 'voice-waked' events
    Failure Indicators: Window not activated, no events emitted, errors in logs
    Evidence: .sisyphus/evidence/task-1-keyword-success.log

  Scenario: [Keyword verification fails with incorrect word]
    Tool: Interactive rust build/test  
    Preconditions: Application is running, configuration has default wake word "小 Shine"
    Steps:
      1. Simulate audio input containing non-wake word ("Hello", "Stop", etc)
      2. Verify energy threshold exceeded to trigger keyword matching process
      3. Confirm ASR processes input and returns non-matching text
      4. Verify window focus does NOT occur when keyword not verified
      5. Check for log messages indicating keyword not recognized
    Expected Result: No window focus, no events, log message indicating mismatch
    Evidence: .sisyphus/evidence/task-1-keyword-failure.log
  ```

  > **Specificity requirements — every scenario MUST use:**
  > - **Selectors**: Specific CSS selectors (`.login-button`, not "the login button")
  > - **Data**: Concrete test data (`"test@example.com"`, not `"[email]"`)
  > - **Assertions**: Exact values (`text contains "Welcome back"`, not "verify it works")
  > - **Timing**: Wait conditions where relevant (`timeout: 10s`)
  > - **Negative**: At least ONE failure/edge case per task
  >
  > **Anti-patterns (your scenario is INVALID if it looks like this):**
  > - ❌ "Verify it works correctly" — HOW? What does "correctly" mean?
  > - ❌ "Check the API returns data" — WHAT data? What fields? What values?
  > - ❌ "Test the component renders" — WHERE? What selector? What content?
  > - ❌ Any scenario without an evidence path

  **Evidence to Capture:**
  - [ ] Each evidence file named: task-{N}-{scenario-slug}.{ext}
  - [ ] Terminal logs showing keyword processing results
  - [ ] Window focus/not-focus based on keyword matching

  **Commit**: YES | NO (groups with N)
  - Message: `feat(voice): Implement keyword spot wake verification`
  - Files: `src-tauri/src/commands/voice_wake.rs`
  - Pre-commit: `cargo check`

- [ ] 2. Modify configuration handling to use wake word field

  **What to do**:
  - Ensure the voice wake loop accesses the configured wake word from AppConfig  
  - Create function to dynamically get current wake word from settings
  - Test that multiple word configurations work properly (Chinese characters, etc)
  - Ensure wake word is loaded only needed instead of on every audio check

  **Must NOT do**:
  - Store hardcoded wake word in code anywhere
  - Reload config on every processed audio segment (performance impact)
  - Break backwards compatibility with existing configs

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `unspecified-high`
    Reason: Moderate complexity dealing with configuration system integration
  - **Skills**: []
    - ``: [Why needed - domain overlap explanation]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 ()
  - **Blocks**: []
  - **Blocked By**: [Task 1] (needs existing voice wake structure modified first)

  **References** (CRITICAL - Be Exhaustive):

  > The executor has NO context from your interview. References are their ONLY guide.
  > Each reference must answer: "What should I look at and WHY?"

  **Pattern References** (existing code to follow):
  - `src-tauri/src/commands/voice_wake.rs:204` - How VOSK URL is accessed from environment variable
  - `src-tauri/src/config.rs:76-81` - AppConfig structure with voice_wake configuration

  **API/Type References** (contracts to implement against):
  - `src-tauri/src/config.rs:191-197` - `get_app_config` Tauri command function

  **Test References** (testing patterns to follow):
  - `src-tauri/src/commands/voice_wake.rs:110` - Example of configuration access with error recovery

  **External References** (libraries and frameworks):
  - [CONFIG REFERENCES]

  **WHY Each Reference Matters** (explain the relevance):
  - Seeing how VOSK url is retrieved to implement same approach for wake word
  - Looking at AppConfig to understand structure for extracting wake word safely

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.
  > Every criterion MUST be verbable by running a command or using a tool.

  **If TDD (tests enabled):**
  - [ ] Add/get functions to access configured wake word in voice wake loop
  - [ ] Tests pass demonstrating proper wake word retrieval from config

  **QA Scenarios (MANDATORY):**

  > **This is NOT optional. A task without QA scenarios WILL BE REJECTED.**
  >
  > Write scenario tests that verify the ACTUAL BEHAVIOR of what you built.
  > Minimum: 1 happy path + 1 failure/edge case per task.
  > Each scenario = exact tool + exact steps + exact assertions + evidence path.
  >
  > **The executing agent MUST run these scenarios after implementation.**
  > **The orchestrator WILL verify evidence files exist before marking task complete.**

  ```
  Scenario: [Configured wake word is properly loaded when app starts]
    Tool: Bash/cli configuration check
    Preconditions: Configuration file contains voice_wake.wake_word with value "小 Shine"
    Steps:
      1. Start the application with logging at verbose level
      2. Trigger the voice wake initialization code path 
      3. Monitor logs to verify application reads correct wake word from config
      4. Check that internal wake comparison uses configured string
    Expected Result: Correct wake word used in keyword comparison logic
    Failure Indicators: Hardcoded default used instead of config value
    Evidence: .sisyphus/evidence/task-2-config-success.log
    
  Scenario: [Different wake words can be configured and work properly]  
    Tool: Configuration modification and testing
    Preconditions: Config has different wake word value
    Steps:
      1. Change wake_word in AppConfig to "Hey Shine"
      2. Restart/refresh voice wake functionality to reload config
      3. Test with both original and new wake word
      4. Verify only new wake word triggers wake sequence
    Expected Result: Configured wake word is used exclusively for activation
    Evidence: .sisyphus/evidence/task-2-config-change.log
  ```

  > **Specificity requirements — every scenario MUST use:**
  > - **Selectors**: Specific CSS selectors (`.login-button`, not "the login button")
  > - **Data**: Concrete test data (`"test@example.com"`, not `"[email]"`)
  > - **Assertions**: Exact values (`text contains "Welcome back"`, not "verify it works")
  > - **Timing**: Wait conditions where relevant (`timeout: 10s`)
  > - **Negative**: At least ONE failure/edge case per task
  >
  > **Anti-patterns (your scenario is INVALID if it looks like this):**
  > - ❌ "Verify it works correctly" — HOW? What does "correctly" mean?
  > - ❌ "Check the API returns data" — WHAT data? What fields? What values?
  > - ❌ "Test the component renders" — WHERE? What selector? What content?
  > - ❌ Any scenario without an evidence path

  **Evidence to Capture:**
  - [ ] Each evidence file named: task-{N}-{scenario-slug}.{ext}
  - [ ] Log captures proving configuration loading works properly
  - [ ] Terminal/stdout outputs showing wake word in use

  **Commit**: YES (with Task 1) | NO  
  - Message: `feat(config): Allow wake word to be read from configuration`
  - Files: `src-tauri/src/commands/voice_wake.rs`, `src-tauri/src/config.rs`
  - Pre-commit: `cargo check`

- [ ] 3. Add keyword spot logging and diagnostics

  **What to do**:
  - Add more detailed logging for keyword matching process
  - Distinguish between "general audio activity" logs and "keyword verified" logs
  - Include timing information for how long keyword matching takes
  - Add logs to distinguish between false triggers vs keyword matches

  **Must NOT do**:
  - Add too much logging that slows down critical audio path  
  - Log personal conversation content beyond just wake word recognition results
  - Include sensitive information in log output

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `quick`
    Reason: Simple logging additions to existing code structure
  - **Skills**: []
    - ``: [Why needed - domain overlap explanation]

  **Parallelization**:
  - **Can Run In Parallel**: YES  
  - **Parallel Group**: Wave 1 ()
  - **Blocks**: []
  - **Blocked By**: [Task 1] (needs the matching functionality to add logging for)

  **References** (CRITICAL - Be Exhaustive):

  > The executor has NO context from your interview. References are their ONLY guide.
  > Each reference must answer: "What should I look at and WHY?"

  **Pattern References** (existing code to follow):
  - `src-tauri/src/commands/voice_wake.rs:220-250` - Current logging patterns for voice wake activities

  **API/Type References** (contracts to implement against):
  - [LOGGING REFERENCES]

  **Test References** (testing patterns to follow):
  - `src-tauri/src/commands/voice_wake.rs:110-130` - Example diagnostic logging patterns

  **External References** (libraries and frameworks):
  - [LOGGING FRAMEWORK REFERENCES]

  **WHY Each Reference Matters** (explain the relevance):
  - Understanding current logging format to add consistent keyword matcher logs
  - Following same patterns for diagnostic messages

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.
  > Every criterion MUST be verbable by running a command or using a tool.

  **If TDD (tests enabled):**
  - [ ] Add new log line indicating when keyword is matched correctly
  - [ ] Add new log line indicating when keyword verification fails
  - [ ] Build test passes with new logging added

  **QA Scenarios (MANDATORY):**

  > **This is NOT optional. A task without QA scenarios WILL BE REJECTED.**
  >
  > Write scenario tests that verify the ACTUAL BEHAVIOR of what you built.
  > Minimum: 1 happy path + 1 failure/edge case per task.
  > Each scenario = exact tool + exact steps + exact assertions + evidence path.
  >
  > **The executing agent MUST run these scenarios after implementation.**
  > **The orchestrator WILL verify evidence files exist before marking task complete.**

  ```
  Scenario: [Proper logging occurs when keyword is correctly matched]
    Tool: Log monitoring
    Preconditions: System has new keyword matching logic installed
    Steps:
      1. Set up log monitoring for voice wake activities
      2. Speak the configured wake word
      3. Confirm "TRUE Wake word detected" or similar special log appears
      4. Verify distinct from simple energy threshold triggers  
    Expected Result: Special log message indicates successful keyword matching
    Failure Indicators: Same log as energy threshold, no distinguishing info
    Evidence: .sisyphus/evidence/task-3-success-log-output.log

  Scenario: [Proper logging occurs when keyword is NOT matched]
    Tool: Log monitoring
    Preconditions: System has new keyword matching logic installed
    Steps:  
      1. Set up log monitoring for voice wake activities
      2. Speak non-wake word that would normally exceed energy threshold
      3. Confirm rejection message is logged
      4. Verify no wake activation despite audio energy trigger  
    Expected Result: Log message indicates wake word not recognized despite audio activity
    Evidence: .sisyphus/evidence/task-3-failure-log-output.log
  ```

  > **Specificity requirements — every scenario MUST use:**
  > - **Selectors**: Specific CSS selectors (`.login-button`, not "the login button")
  > - **Data**: Concrete test data (`"test@example.com"`, not `"[email]"`)
  > - **Assertions**: Exact values (`text contains "Welcome back"`, not "verify it works")
  > - **Timing**: Wait conditions where relevant (`timeout: 10s`)
  > - **Negative**: At least ONE failure/edge case per task
  >
  > **Anti-patterns (your scenario is INVALID if it looks like this):**
  > - ❌ "Verify it works correctly" — HOW? What does "correctly" mean?
  > - ❌ "Check the API returns data" — WHAT data? What fields? What values?
  > - ❌ "Test the component renders" — WHERE? What selector? What content?
  > - ❌ Any scenario without an evidence path

  **Evidence to Capture:**
  - [ ] Each evidence file named: task-{N}-{scenario-slug}.{ext}
  - [ ] Log outputs showing new diagnostic information

  **Commit**: YES (with Task 1 & 2) | NO
  - Message: `chore(logging): Improve keyword matching diagnostics`  
  - Files: `src-tauri/src/commands/voice_wake.rs`
  - Pre-commit: `cargo check`

- [ ] 4. Integrate ASR processing with keyword verification

  **What to do**:
  - Connect voice wake loop to VOSK service for quick transcription
  - Process audio buffer through ASR to get text representation of what was said
  - Compare ASR result with configured wake word
  - Implement timeout logic to prevent hanging on ASR processing
  - Create reusable ASR function for potential future enhancements

  **Must NOT do**:
  - Route all audio through ASR continuously (performance and privacy)
  - Hang the main audio processing loop if ASR service is unavailable
  - Expose conversation content beyond wake word verification

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `deep`
    Reason: Complex async integration between audio capture, ASR processing, and keyword matching
  - **Skills**: []
    - ``: [Why needed - domain overlap explanation]

  **Parallelization**:
  - **Can Run In Parallel**: YES 
  - **Parallel Group**: Wave 2 ()
  - **Blocks**: []
  - **Blocked By**: [Task 2 & Task 1] (needs configuration access and basic structure)

  **References** (CRITICAL - Be Exhaustive):

  > The executor has NO context from your interview. References are their ONLY guide.
  > Each reference must answer: "What should I look at and WHY?"

  **Pattern References** (existing code to follow):
  - `src-tauri/src/commands/voice_wake.rs:342-426` - run_vosk_client function shows current ASR processing
  - `src-tauri/src/voice/*` - Other voice processing modules to understand integration patterns

  **API/Type References** (contracts to implement against):
  - `tokio_tungstenite` and `futures_util` libraries already in use for WebSocket connections
  - `src-tauri/src/config.rs#VoskConfig` for server connection details

  **Test References** (testing patterns to follow):
  - [TEST PATTERNS]

  **External References** (libraries and frameworks):
  - https://github.com/robiot/vosk-rs - Vosk-rs for speech recognition

  **WHY Each Reference Matters** (explain the relevance):
  - Understanding existing ASR connection patterns to create lightweight version for quick verification
  - Seeing how WebSocket communication works to implement efficient keyword checking

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.
  > Every criterion MUST be vervable by running a command or using a tool.

  **If TDD (tests enabled):**
  - [ ] Implement perform_keyword_matching function that connects to VOSK temporarily  
  - [ ] Test with mock ASR responses showing different recognition results

  **QA Scenarios (MANDATORY):**

  > **This is NOT optional. A task without QA scenarios WILL BE REJECTED.**
  >
  > Write scenario tests that verify the ACTUAL BEHAVIOR of what you built.
  > Minimum: 1 happy path + 1 failure/edge case per task.
  > Each scenario = exact tool + exact steps + exact assertions + evidence path.
  >
  > **The executing agent MUST run these scenarios after implementation.**
  > **The orchestrator WILL verify evidence files exist before marking task complete.**

  ```
  Scenario: [ASR properly connects and verifies wake word when speaking to mic]
    Tool: Interactive build/run verification
    Preconditions: VOSK server running at configured websocket URL
    Steps:
      1. Start audio processing with keyword verification enabled
      2. Make loud sound that exceeds energy threshold  
      3. Speak configured wake word into microphone
      4. Verify ASR connection succeeds and recognizes wake phrase
      5. Confirm wake sequence is triggered correctly
    Expected Result: ASR processes speech and confirms wake word match
    Failure Indicators: Connection fails, timeout, or non-match
    Evidence: .sisyphus/evidence/task-4-asr-connection-success.log

  Scenario: [ASR handles server errors gracefully during keyword verification]
    Tool: Server interruption test
    Preconditions: Configured VOSK server running initially
    Steps:
      1. Start audio processing and trigger energy threshold
      2. Stop VOSK server mid-process to create connection issue
      3. Verify application continues running without freezing
      4. Confirm appropriate error handling and fallback behavior
    Expected Result: Clean error handling without crashing audio pipeline 
    Evidence: .sisyphus/evidence/task-4-asr-error-handling.log
  ```

  > **Specificity requirements — every scenario MUST use:**
  > - **Selectors**: Specific CSS selectors (`.login-button`, not "the login button")
  > - **Data**: Concrete test data (`"test@example.com"`, not `"[email]"`)
  > - **Assertions**: Exact values (`text contains "Welcome back"`, not "verify it works")
  > - **Timing**: Wait conditions where relevant (`timeout: 10s`)
  > - **Negative**: At least ONE failure/edge case per task
  >
  > **Anti-patterns (your scenario is INVALID if it looks like this):**
  > - ❌ "Verify it works correctly" — HOW? What does "correctly" mean?
  > - ❌ "Check the API returns data" — WHAT data? What fields? What values?
  > - ❌ "Test the component renders" — WHERE? What selector? What content?
  > - ❌ Any scenario without an evidence path

  **Evidence to Capture:**
  - [ ] Each evidence file named: task-{N}-{scenario-slug}.{ext}
  - [ ] Network traffic logs, connection status reports
  - [ ] ASR transcription results and matching outcomes

  **Commit**: YES | NO
  - Message: `feat(asr): Integrate keyword verification with VOSK`
  - Files: `src-tauri/src/commands/voice_wake.rs`
  - Pre-commit: `cargo check`

- [ ] 5. Update UI emission triggers to require keyword match

  **What to do**:
  - Modify event emission logic so that `"voice-waked"` and other UI events only happen after keyword verification
  - Keep all current UI integration but gate behind successful keyword match
  - Update window focusing and TTS response to happen only after ASR confirms wake word
  - Verify window focus, TTS, and UI state changes still work properly

  **Must NOT do**:
  - Break existing UI flow that depends on voice events
  - Stop emitting events that are needed for other parts of system
  - Reduce responsiveness unnecessarily after keyword verification

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `deep`  
    Reason: Critical UI functionality needs careful verification and testing
  - **Skills**: []
    - ``: [Why needed - domain overlap explanation]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 ()
  - **Blocks**: []
  - **Blocked By**: [Task 4] (needs ASR integration first)

  **References** (CRITICAL - Be Exhaustive):

  > The executor has NO context from your interview. References are their ONLY guide.
  > Each reference must answer: "What should I look at and WHY?"

  **Pattern References** (existing code to follow):
  - `src-tauri/src/commands/voice_wake.rs:248-278` - Current UI event emission patterns

  **API/Type References** (contracts to implement against):
  - `AppHandle.emit_all` method already in use for event dispatch

  **Test References** (testing patterns to follow):
  - [UI EVENT REFERENCES]

  **External References** (libraries and frameworks):
  - [TAURI EVENT SYSTEM REFERENCES]

  **WHY Each Reference Matters** (explain the relevance):
  - Seeing exact current patterns to preserve functionality while adding keyword filter

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.
  > Every criterion MUST be verbable by running a command or using a tool.

  **If TDD (tests enabled):**
  - [ ] Modify event emissions to only happen after keyword verification
  - [ ] Build and test to ensure UI still responds properly to valid wake words

  **QA Scenarios (MANDATORY):**

  > **This is NOT optional. A task without QA scenarios WILL BE REJECTED.**
  >
  > Write scenario tests that verify the ACTUAL BEHAVIOR of what you built.
  > Minimum: 1 happy path + 1 failure/edge case per task.
  > Each scenario = exact tool + exact steps + exact assertions + evidence path.
  >
  > **The executing agent MUST run these scenarios after implementation.**
  > **The orchestrator WILL verify evidence files exist before marking task complete.**

  ```
  Scenario: [UI state changes only when keyword is successfully recognized]
    Tool: Interactive event monitoring
    Preconditions: Keyword recognition integrated and operational
    Steps:
      1. Start app and set up monitoring for "voice-waked", "voice-state-changed" events
      2. Produce loud noise to trigger energy threshold but not say wake word
      3. Verify no UI events fired (but energy triggered logging)
      4. Speak configured wake word and verify UI events fire properly  
    Expected Result: UI events only happen after keyword verification, not after energy threshold
    Failure Indicators: Events fire on loud sound, no events on actual wake word
    Evidence: .sisyphus/evidence/task-5-event-verification.log

  Scenario: [Window focus and TTS work properly after keyword verification]
    Tool: Window activity monitoring
    Preconditions: Correct keyword verification and window control functionality  
    Steps:
      1. Say non-wake word that exceeds energy threshold - no window focus
      2. Say configured wake word - verify window comes to front and TTS plays
      3. Check that TTS plays proper wake response after keyword match confirmation
    Expected Result: All UI responses occur only after keyword verification
    Evidence: .sisyphus/evidence/task-5-ui-verification.log
  ```  

  > **Specificity requirements — every scenario MUST use:**
  > - **Selectors**: Specific CSS selectors (`.login-button`, not "the login button")
  > - **Data**: Concrete test data (`"test@example.com"`, not `"[email]"`)
  > - **Assertions**: Exact values (`text contains "Welcome back"`, not "verify it works")
  > - **Timing**: Wait conditions where relevant (`timeout: 10s`)
  > - **Negative**: At least ONE failure/edge case per task
  >
  > **Anti-patterns (your scenario is INVALID if it looks like this):**
  > - ❌ "Verify it works correctly" — HOW? What does "correctly" mean?
  > - ❌ "Check the API returns data" — WHAT data? What fields? What values?
  > - ❌ "Test the component renders" — WHERE? What selector? What content?
  > - ❌ Any scenario without an evidence path

  **Evidence to Capture:**
  - [ ] Each evidence file named: task-{N}-{scenario-slug}.{ext}
  - [ ] Event emission logs, UI behavior captures

  **Commit**: YES | NO
  - Message: `fix(ui-events): Gate UI emissions behind keyword verification`  
  - Files: `src-tauri/src/commands/voice_wake.rs`
  - Pre-commit: `cargo check`

- [ ] 6. Add audio buffering for keyword verification capture

  **What to do**:
  - Implement audio buffer to collect segment of audio that preceded wake energy threshold trigger
  - Store last ~2 seconds of audio when using keyword verification
  - Ensure buffer efficiently utilizes existing audio rx channel and doesn't interfere with main processing
  - Manage memory efficiently to prevent growing buffers indefinitely

  **Must NOT do**:
  - Store audio content indefinitely (privacy concerns)
  - Impact responsiveness of main audio processing loop
  - Interfere with existing ASR streaming functionality

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `unspecified-high`
    Reason: Requires understanding of CPAL audio processing and threading
  - **Skills**: []
    - ``: [Why needed - domain overlap explanation]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 ()  
  - **Blocks**: []
  - **Blocked By**: [Task 1] (needs basic structure modified first)

  **References** (CRITICAL - Be Exhaustive):

  > The executor has NO context from your interview. References are their ONLY guide.
  > Each reference must answer: "What should I look at and WHY?"

  **Pattern References** (existing code to follow):
  - `src-tauri/src/commands/voice_wake.rs:146-338` - Main audio loop with audio_rx channel
  - `src-tauri/src/voice/audio_capture.rs` - Audio buffering and processing patterns

  **API/Type References** (contracts to implement against):
  - `std::sync::mpsc::Receiver<Vec<f32>>` from current audio implementation

  **Test References** (testing patterns to follow):
  - [AUDIO BUFFER PATTERNS]

  **External References** (libraries and frameworks):
  - [CPAL AUDIO PROCESSING REFERENCES]

  **WHY Each Reference Matters** (explain the relevance):
  - Understanding existing audio handling to implement buffer integration without interference
  - Learning proper patterns for working with audio channels efficiently

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.
  > Every criterion MUST be vervable by running a command or using a tool.

  **If TDD (tests enabled):**
  - [ ] Add audio buffer implementation for capturing potential wake word audio
  - [ ] Verify buffer efficiently manages memory and stores last N seconds
  - [ ] Ensure no performance degradation of primary voice wake loop

  **QA Scenarios (MANDATORY):**

  > **This is NOT optional. A task without QA scenarios WILL BE REJECTED.**
  >
  > Write scenario tests that verify the ACTUAL BEHAVIOR of what you built.
  > Minimum: 1 happy path + 1 failure/edge case per task.
  > Each scenario = exact tool + exact steps + exact assertions + evidence path.
  >
  > **The executing agent MUST run these scenarios after implementation.**
  > **The orchestrator WILL verify evidence files exist before marking task complete.**

  ```
  Scenario: [Audio buffer captures preceding audio for keyword verification]
    Tool: Audio data analysis  
    Preconditions: Audio buffering implemented alongside keyword verification
    Steps:
      1. Allow audio processing to run with buffering enabled
      2. Create loud sound that triggers wake processing and captures audio to buffer
      3. Extract and verify the buffered audio represents content before trigger
      4. Confirm buffer operates efficiently without memory leaks
    Expected Result: Audio buffer captures relevant pre-trigger audio
    Failure Indicators: No audio in buffer, incorrect timing, memory issues
    Evidence: .sisyphus/evidence/task-6-audio-buffer-success.log

  Scenario: [Buffer memory consumption remains reasonable during extended operation]
    Tool: Memory monitoring
    Preconditions: Audio buffering running for extended period
    Steps:
      1. Monitor memory usage during extended audio processing (10+ minutes)
      2. Verify audio buffer does not expand unlimited during silence
      3. Confirm buffer clears appropriately and doesn't retain old data
    Expected Result: Stable memory usage with appropriate buffer rotation
    Evidence: .sisyphus/evidence/task-6-memory-monitoring.log
  ```

  > **Specificity requirements — every scenario MUST use:**
  > - **Selectors**: Specific CSS selectors (`.login-button`, not "the login button")
  > - **Data**: Concrete test data (`"test@example.com"`, not `"[email]"`)
  > - **Assertions**: Exact values (`text contains "Welcome back"`, not "verify it works")
  > - **Timing**: Wait conditions where relevant (`timeout: 10s`)
  > - **Negative**: At least ONE failure/edge case per task
  >
  > **Anti-patterns (your scenario is INVALID if it looks like this):**
  > - ❌ "Verify it works correctly" — HOW? What does "correctly" mean?
  > - ❌ "Check the API returns data" — WHAT data? What fields? What values?
  > - ❌ "Test the component renders" — WHERE? What selector? What content?
  > - ❌ Any scenario without an evidence path

  **Evidence to Capture:**
  - [ ] Each evidence file named: task-{N}-{scenario-slug}.{ext}
  - [ ] Memory usage metrics, buffer size validation

  **Commit**: YES | NO
  - Message: `feat(buffer): Add audio buffering for keyword verification`  
  - Files: `src-tauri/src/commands/voice_wake.rs`
  - Pre-commit: `cargo check`

- [ ] 7. Create keyword matching utility functions

  **What to do**:
  - Build reusable functions/classes for keyword matching functionality
  - Implement fuzzy matching for handling common pronunciation variations
  - Add utility functions for comparing ASR results to expected wake words
  - Include utilities for processing ASR response formats from VOSK

  **Must NOT do**:
  - Create overly complex matching algorithms that slow down recognition
  - Store historical data about user's pronunciations permanently
  - Implement matching that's too permissive and matches random phrases

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `visual-engineering`
    Reason: Creating well-designed utility functions with good API design
  - **Skills**: []
    - ``: [Why needed - domain overlap explanation]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 ()
  - **Blocks**: []
  - **Blocked By**: [Task 4] (needs ASR processing pattern to understand result format)

  **References** (CRITICAL - Be Exhaustive):

  > The executor has NO context from your interview. References are their ONLY guide.
  > Each reference must answer: "What should I look at and WHY?"

  **Pattern References** (existing code to follow):
  - `src-tauri/src/commands/voice_wake.rs:366-379` - Current ASR result parsing 
  - `src/voice/asr_client.rs` if exists - Other ASR processing patterns

  **API/Type References** (contracts to implement against):
  - `serde_json::Value` for processing ASR responses  
  - String comparison functions for matching logic

  **Test References** (testing patterns to follow):
  - [STRING MATCHING PATTERNS]

  **External References** (libraries and frameworks):
  - [TEXT PROCESSING REFERENCES]

  **WHY Each Reference Matters** (explain the relevance):
  - Seeing current ASR result structure to properly extract and match text
  - Understanding how text is represented in responses

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.
  > Every criterion MUST be verbable by running a command or using a tool.

  **If TDD (tests enabled):**
  - [ ] Add utility functions for comparing wake words  
  - [ ] Implement fuzzy matching algorithm with adjustable tolerance
  - [ ] Test utility function correctness with various inputs

  **QA Scenarios (MANDATORY):**

  > **This is NOT optional. A task without QA scenarios WILL BE REJECTED.**
  >
  > Write scenario tests that verify the ACTUAL BEHAVIOR of what you built.
  > Minimum: 1 happy path + 1 failure/edge case per task.
  > Each scenario = exact tool + exact steps + exact assertions + evidence path.
  >
  > **The executing agent MUST run these scenarios after implementation.**
  > **The orchestrator WILL verify evidence files exist before marking task complete.**

  ```
  Scenario: [Utility functions accurately match wake word with normal pronunciation]
    Tool: Unit testing with various text comparisons
    Preconditions: Keyword matching utilities implemented
    Steps:
      1. Call keyword matching utility with perfect match: "小 Shine" vs "小 Shine"
      2. Call utility with slight punctuation differences: "小 Shine" vs "小 Shine!"
      3. Verify both return positive match results
      4. Confirm false negatives with different phrases  
    Expected Result: Utilities return correct matches for same/very similar phrases
    Failure Indicators: False positives with dissimilar phrases, false negatives with same
    Evidence: .sisyphus/evidence/task-7-utility-accurate-results.log

  Scenario: [Fuzzy matching tolerates minor pronunciation differences]
    Tool: Unit tests with variations
    Preconditions: Fuzzy matching utilities enabled
    Steps:
      1. Test with slight phonetic difference: "Shine" vs "Shiene" (transposed letters)
      2. Test with accent/tonal variation: "小 Shine" vs "小 shīne"
      3. Confirm tolerance thresholds are reasonable
    Expected Result: Minor variations match, major differences don't
    Evidence: .sisyphus/evidence/task-7-fuzzy-matching.log
  ```

  > **Specificity requirements — every scenario MUST use:**
  > - **Selectors**: Specific CSS selectors (`.login-button`, not "the login button")
  > - **Data**: Concrete test data (`"test@example.com"`, not `"[email]"`)
  > - **Assertions**: Exact values (`text contains "Welcome back"`, not "verify it works")
  > - **Timing**: Wait conditions where relevant (`timeout: 10s`)
  > - **Negative**: At least ONE failure/edge case per task
  >
  > **Anti-patterns (your scenario is INVALID if it looks like this):**
  > - ❌ "Verify it works correctly" — HOW? What does "correctly" mean?
  > - ❌ "Check the API returns data" — WHAT data? What fields? What values?
  > - ❌ "Test the component renders" — WHERE? What selector? What content?
  > - ❌ Any scenario without an evidence path

  **Evidence to Capture:**
  - [ ] Each evidence file named: task-{N}-{scenario-slug}.{ext}
  - [ ] Unit test results, utility function verification

  **Commit**: YES | NO
  - Message: `feat(utils): Add keyword matching utilities`  
  - Files: `src-tauri/src/commands/voice_wake.rs`
  - Pre-commit: `cargo check`

- [ ] 8. Update error handling for ASR processing

  **What to do**:
  - Add proper error handling to ASR processing for keyword verification
  - Implement timeouts to prevent hanging when VOSK server is unresponsive
  - Create fallback behavior if ASR server is unreachable (possibly revert to partial energy method)
  - Log appropriate messages when ASR fails but energy threshold was met

  **Must NOT do**:
  - Let ASR processing errors crash main audio loop
  - Block audio processing waiting indefinitely for ASR results
  - Expose sensitive error details through logging

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `deep`
    Reason: Complex error handling integration with async ASR processing
  - **Skills**: []
    - ``: [Why needed - domain overlap explanation]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 ()
  - **Blocks**: []
  - **Blocked By**: [Task 4] (needs ASR integration first)

  **References** (CRITICAL - Be Exhaustive):

  > The executor has NO context from your interview. References are their ONLY guide.
  > Each reference must answer: "What should I look at and WHY?"

  **Pattern References** (existing code to follow):
  - `src-tauri/src/commands/voice_wake.rs:350-426` - run_vosk_client error handling patterns

  **API/Type References** (contracts to implement against):
  - Current error handling in voice wake loop functions

  **Test References** (testing patterns to follow):
  - [ERROR HANDLING PATTERNS] 

  **External References** (libraries and frameworks):
  - [ASYNC ERROR HANDLING REFERENCES]

  **WHY Each Reference Matters** (explain the relevance):
  - Learning current error handling style to maintain consistency

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.
  > Every criterion MUST be verbable by running a command or using a tool.

  **If TDD (tests enabled):**
  - [ ] Add timeout handling for ASR processing during keyword verification
  - [ ] Implement proper error propagation and fallback handling
  - [ ] Verify main audio loop continues to function if ASR fails

  **QA Scenarios (MANDATORY):**

  > **This is NOT optional. A task without QA scenarios WILL BE REJECTED.**
  >
  > Write scenario tests that verify the ACTUAL BEHAVIOR of what you built.
  > Minimum: 1 happy path + 1 failure/edge case per task.
  > Each scenario = exact tool + exact steps + exact assertions + evidence path.
  >
  > **The executing agent MUST run these scenarios after implementation.**
  > **The orchestrator WILL verify evidence files exist before marking task complete.**

  ```
  Scenario: [Application handles VOSK server unresponsiveness gracefully]  
    Tool: Server interruption and timeout verification
    Preconditions: Keyword verification with ASR enabled
    Steps:
      1. Start audio processing with keyword verification active
      2. Prevent VOSK server from responding to create timeout condition
      3. Trigger energy threshold in audio input
      4. Confirm application doesn't hang and continues processing
      5. Check appropriate error messages in logs
    Expected Result: System continues running, no hangs on unresponsive ASR
    Failure Indicators: Application freeze, crash, or exception  
    Evidence: .sisyphus/evidence/task-8-timeout-handling.log

  Scenario: [Fallback behavior activates when ASR unavailable]
    Tool: Server unavailable simulation
    Preconditions: VOSK server stopped/cannot connect
    Steps:
      1. Configure system to use ASR-based keyword verification
      2. Stop VOSK server to make it unreachable
      3. Test audio input and observe how system handles unavailability
      4. Verify graceful degradation behavior
    Expected Result: Either notification to user or fallback mechanism
    Evidence: .sisyphus/evidence/task-8-fallback-handling.log
  ```

  > **Specificity requirements — every scenario MUST use:**
  > - **Selectors**: Specific CSS selectors (`.login-button`, not "the login button")
  > - **Data**: Concrete test data (`"test@example.com"`, not `"[email]"`)
  > - **Assertions**: Exact values (`text contains "Welcome back"`, not "verify it works")
  > - **Timing**: Wait conditions where relevant (`timeout: 10s`)
  > - **Negative**: At least ONE failure/edge case per task
  >
  > **Anti-patterns (your scenario is INVALID if it looks like this):**
  > - ❌ "Verify it works correctly" — HOW? What does "correctly" mean?
  > - ❌ "Check the API returns data" — WHAT data? What fields? What values?
  > - ❌ "Test the component renders" — WHERE? What selector? What content?
  > - ❌ Any scenario without an evidence path

  **Evidence to Capture:**
  - [ ] Each evidence file named: task-{N}-{scenario-slug}.{ext}
  - [ ] Error logs, timeout responses, fallback behavior evidence

  **Commit**: YES | NO
  - Message: `fix(error-handling): Improve ASR error resilience`  
  - Files: `src-tauri/src/commands/voice_wake.rs`
  - Pre-commit: `cargo check`

- [ ] 9. Test wake word variations and configuration changes

  **What to do**:
  - Test with multiple different wake words to confirm functionality is not hardcoded to defaults
  - Verify that configuration changes to the wake word work properly
  - Test with Chinese characters, mixed English-Chinese, and other variations
  - Ensure changing configuration dynamically affects recognition

  **Must NOT do**:
  - Assume only default wake word will be used
  - Break support for unicode/multibyte characters
  - Fail to update recognition engine when configs change
  
  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `quick`
    Reason: Functional test of configuration integration
  - **Skills**: []
    - ``: [Why needed - domain overlap explanation]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 ()
  - **Blocks**: []
  - **Blocked By**: [Task 2] (needs configuration integration)

  **References** (CRITICAL - Be Exhaustive):

  > The executor has NO context from your interview. References are their ONLY guide.
  > Each reference must answer: "What should I look at and WHY?"

  **Pattern References** (existing code to follow):
  - `src-tauri/src/config.rs:46-56` - VoiceWakeConfig default values

  **API/Type References** (contracts to implement against):
  - AppConfig structure with configurable wake_word field

  **Test References** (testing patterns to follow):
  - [CONFIG TEST PATTERNS]

  **External References** (libraries and frameworks):
  - [CHARACTER SET HANDLING]

  **WHY Each Reference Matters** (explain the relevance):
  - Seeing example wake words used to know what types of character combinations to test

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.
  > Every criterion MUST be vervable by running a command or using a tool.

  **If TDD (tests enabled):**
  - [ ] Test with various configured wake words
  - [ ] Verify non-default configurations work properly for keyword matching

  **QA Scenarios (MANDATORY):**

  > **This is NOT optional. A task without QA scenarios WILL BE REJECTED.**
  >
  > Write scenario tests that verify the ACTUAL BEHAVIOR of what you built.
  > Minimum: 1 happy path + 1 failure/edge case per task.
  > Each scenario = exact tool + exact steps + exact assertions + evidence path.
  >
  > **The executing agent MUST run these scenarios after implementation.**
  > **The orchestrator WILL verify evidence files exist before marking task complete.**

  ```
  Scenario: [Chinese-character wake words properly detected after config change]
    Tool: Configuration update and keyword verification test
    Preconditions: Application with configurable wake word settings
    Steps:
      1. Change configured wake word to different Chinese phrase: "你好助手"
      2. Confirm system uses new configuration in wake detection
      3. Speak new wake word into microphone
      4. Verify keyword matching works with updated configuration
    Expected Result: Different wake word from config is properly recognized
    Failure Indicators: Old default continues to work, new one doesn't
    Evidence: .sisyphus/evidence/task-9-chinese-chars.log

  Scenario: [Mixed-language wake words work properly]
    Tool: Mixed-language keyword test
    Steps:
      1. Configure wake word with mixed English and Chinese: "小助手Hey"
      2. Test with spoken version of mixed phrase
      3. Confirm ASR properly handles multilingual input
    Expected Result: Multilingual wake words are recognized correctly
    Evidence: .sisyphus/evidence/task-9-mixed-language.log
  ```

  > **Specificity requirements — every scenario MUST use:**
  > - **Selectors**: Specific CSS selectors (`.login-button`, not "the login button")
  > - **Data**: Concrete test data (`"test@example.com"`, not `"[email]"`)
  > - **Assertions**: Exact values (`text contains "Welcome back"`, not "verify it works")
  > - **Timing**: Wait conditions where relevant (`timeout: 10s`)
  > - **Negative**: At least ONE failure/edge case per task
  >
  > **Anti-patterns (your scenario is INVALID if it looks like this):**
  > - ❌ "Verify it works correctly" — HOW? What does "correctly" mean?
  > - ❌ "Check the API returns data" — WHAT data? What fields? What values?
  > - ❌ "Test the component renders" — WHERE? What selector? What content?
  > - ❌ Any scenario without an evidence path

  **Evidence to Capture:**
  - [ ] Each evidence file named: task-{N}-{scenario-slug}.{ext}
  - [ ] Configuration test results, multilingual character handling evidence

  **Commit**: YES (with previous tasks) | NO
  - Message: `test(variants): Test wake word variations and config changes`  
  - Files: `multiple`
  - Pre-commit: `cargo check`

- [ ] 10. Integration test with manual verification

  **What to do**:
  - Full end-to-end test of the keyword spot wake functionality
  - Verify real-world usage scenarios work as expected
  - Manually test that only configured wake word activates the system
  - Verify all previous functionality still works correctly (window focus, UI, etc)

  **Must NOT do**:
  - Skip manual verification even if individual components test well
  - Forget to test in real operating environment with actual microphone
  - Miss verification of full workflow from trigger to UI activation

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `deep`
    Reason: Comprehensive end-to-end testing of audio processing and application behavior
  - **Skills**: []
    - ``: [Why needed - domain overlap explanation]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 4 (Sequential)
  - **Blocks**: []
  - **Blocked By**: [Tasks 1-9] (needs all components integrated)

  **References** (CRITICAL - Be Exhaustive):

  > The executor has NO context from your interview. References are their ONLY guide.
  > Each reference must answer: "What should I look at and WHY?"

  **Pattern References** (existing code to follow):
  - All previously implemented tasks and their verification scenarios

  **API/Type References** (contracts to implement against):
  - Complete voice wake functionality integration

  **Test References** (testing patterns to follow):
  - Pre/post integration behavior verification patterns 

  **External References** (libraries and frameworks):
  - [REAL-TIME AUDIO TESTING PATTERNS]

  **WHY Each Reference Matters** (explain the relevance):
  - Confirming all components work together in realistic scenario

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.
  > Every criterion MUST be verbable by running a command or using a tool.

  **If TDD (tests enabled):**
  - [ ] Execute complete workflow test demonstrating keyword spot wake
  - [ ] Validate all integrated components working properly together

  **QA Scenarios (MANDATORY):**

  > **This is NOT optional. A task without QA scenarios WILL BE REJECTED.**
  >
  > Write scenario tests that verify the ACTUAL BEHAVIOR of what you built.
  > Minimum: 1 happy path + 1 failure/edge case per task.
  > Each scenario = exact tool + exact steps + exact assertions + evidence path.
  >
  > **The executing agent MUST run these scenarios after implementation.**
  > **The orchestrator WILL verify evidence files exist before marking task complete.**

  ```
  Scenario: [End-to-end keyword spot wake works in real world usage]
    Tool: Manual end-to-end test with actual voice input
    Preconditions: All keyword spot functionality implemented and deployed locally
    Steps:
      1. Start application in normal running state
      2. Speak configured wake word (e.g. "小 Shine") to microphone  
      3. Verify app responds by bringing window to front and playing TTS
      4. Try with other loud sounds/words and confirm those don't activate
    Expected Result: System only responds to configured wake word, not other loud sounds
    Failure Indicators: Non-wake sounds trigger activation, wake word doesn't work
    Evidence: .sisyphus/evidence/task-10-e2e-success.log

  Scenario: [Window focus and UI events work properly after end-to-end integration]
    Tool: UI behavior verification
    Preconditions: Full system running with keyword verification active
    Steps:
      1. Configure wake word and speak to trigger recognition 
      2. Verify window focus happens correctly (app window becomes active)
      3. Confirm UI state updates to listening mode properly
      4. Check all connected UI events fire as expected
    Expected Result: Complete UI flow functions as before but with keyword verification
    Evidence: .sisyphus/evidence/task-10-ui-integration.log
  ```

  > **Specificity requirements — every scenario MUST use:**
  > - **Selectors**: Specific CSS selectors (`.login-button`, not "the login button")
  > - **Data**: Concrete test data (`"test@example.com"`, not `"[email]"`)
  > - **Assertions**: Exact values (`text contains "Welcome back"`, not "verify it works")
  > - **Timing**: Wait conditions where relevant (`timeout: 10s`)
  > - **Negative**: At least ONE failure/edge case per task
  >
  > **Anti-patterns (your scenario is INVALID if it looks like this):**
  > - ❌ "Verify it works correctly" — HOW? What does "correctly" mean?
  > - ❌ "Check the API returns data" — WHAT data? What fields? What values?
  > - ❌ "Test the component renders" — WHERE? What selector? What content?
  > - ❌ Any scenario without an evidence path

  **Evidence to Capture:**
  - [ ] Each evidence file named: task-{N}-{scenario-slug}.{ext}
  - [ ] Complete manual testing logs, video/capture of working flow

  **Commit**: YES | NO
  - Message: `test(integration): End-to-end keyword spot wake verification`  
  - Files: `all affected components`
  - Pre-commit: `cargo check`

- [ ] 11. Performance verification

  **What to do**:
  - Measure performance impact of keyword verification vs simple energy detection
  - Confirm that new functionality doesn't slow down wake response time significantly
  - Monitor CPU/memory usage during keyword verification activities
  - Document performance benchmarks for new implementation

  **Must NOT do**:
  - Ignore performance considerations thinking they aren't important
  - Create solution that is so inefficient it impacts audio responsiveness
  - Skip measuring actual impacts of new features

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `visual-engineering`
    Reason: Analysis and benchmarking with quantitative results
  - **Skills**: []
    - ``: [Why needed - domain overlap explanation]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 4 (before final verification)
  - **Blocks**: []
  - **Blocked By**: [Task 1] (basic implementation needed for measurement)

  **References** (CRITICAL - Be Exhaustive):

  > The executor has NO context from your interview. References are their ONLY guide.
  > Each reference must answer: "What should I look at and WHY?"

  **Pattern References** (existing code to follow):
  - [PERFORMANCE BENCHMARK PATTERNS]

  **API/Type References** (contracts to implement against):
  - [PERFORMANCE METRIC REFERENCES]

  **Test References** (testing patterns to follow):
  - [RESOURCE MONITORING PATTERNS]

  **External References** (libraries and frameworks):
  - [PERFORMANCE MEASUREMENT TOOLS]

  **WHY Each Reference Matters** (explain the relevance):
  - Establishing baseline for performance impact assessment

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.
  > Every criterion MUST be verbable by running a command or using a tool.

  **If TDD (tests enabled):**
  - [ ] Measure and document performance impact of keyword verification
  - [ ] Confirm response times remain acceptable after new functionality

  **QA Scenarios (MANDATORY):**

  > **This is NOT optional. A task without QA scenarios WILL BE REJECTED.**
  >
  > Write scenario tests that verify the ACTUAL BEHAVIOR of what you built.
  > Minimum: 1 happy path + 1 performance-related edge case per task.
  > Each scenario = exact tool + exact steps + exact assertions + evidence path.
  >
  > **The executing agent MUST run these scenarios after implementation.**
  > **The orchestrator WILL verify evidence files exist before marking task complete.**

  ```
  Scenario: [Keyword verification doesn't significantly slow down wake response]
    Tool: Performance timing measurement
    Preconditions: Keyword verification implemented and active
    Steps:
      1. Time the response from energy threshold to actual wake action
      2. Compare performance to baseline without keyword verification
      3. Verify response stays under acceptable threshold (e.g. <2 seconds)
      4. Monitor for processing delays in audio loop
    Expected Result: Minimal performance degradation compared to original implementation
    Failure Indicators: Significant delay between trigger and response activation  
    Evidence: .sisyphus/evidence/task-11-performance-timing.log

  Scenario: [Resource usage remains reasonable during extended operation]
    Tool: Resource monitoring
    Preconditions: System running keyword verification continuously
    Steps:
      1. Monitor CPU and memory usage during keyword verification processing
      2. Run extended test (>30 mins) to look for resource leaks
      3. Monitor during rapid-fire wake attempts 
    Expected Result: Acceptable CPU and memory usage during normal operation
    Evidence: .sisyphus/evidence/task-11-resource-usage.log
  ```

  > **Specificity requirements — every scenario MUST use:**
  > - **Selectors**: Specific CSS selectors (`.login-button`, not "the login button")
  > - **Data**: Concrete test data (`"test@example.com"`, not `"[email]"`)
  > - **Assertions**: Exact values (`text contains "Welcome back"`, not "verify it works")
  > - **Timing**: Wait conditions where relevant (`timeout: 10s`)
  > - **Negative**: At least ONE failure/edge case per task
  >
  > **Anti-patterns (your scenario is INVALID if it looks like this):**
  > - ❌ "Verify it works correctly" — HOW? What does "correctly" mean?
  > - ❌ "Check the API returns data" — WHAT data? What fields? What values?
  > - ❌ "Test the component renders" — WHERE? What selector? What content?
  > - ❌ Any scenario without an evidence path

  **Evidence to Capture:**
  - [ ] Each evidence file named: task-{N}-{scenario-slug}.{ext}
  - [ ] Performance benchmarks, resource usage metrics

  **Commit**: YES | NO
  - Message: `perf: Optimize keyword verification for minimal impact`  
  - Files: `src-tauri/src/commands/voice_wake.rs` (optimizations)
  - Pre-commit: `cargo check`

- [ ] 12. Documentation update

  **What to do**:
  - Update code comments to reflect new keyword verification functionality
  - Add documentation explaining the hybrid energy+keyword matching approach  
  - Update any configuration documentation to explain new keyword features
  - Include troubleshooting info for keyword verification issues

  **Must NOT do**:
  - Leave comments inconsistent with new implementation
  - Skip documenting important configuration aspects for other developers
  - Forget to update any user guides that mention the old behavior

  **Recommended Agent Profile**:
  > Select category + skills based on task domain. Justify each choice.
  - **Category**: `writing`
    Reason: Content-focused documentation writing task
  - **Skills**: []
    - ``: [Why needed - domain overlap explanation]

  **Parallelization**:
  - **Can Run In Parallel**: NO (final documentation step)
  - **Parallel Group**: Wave 4 (after integration)
  - **Blocks**: []
  - **Blocked By**: [All previous tasks] (needs full understanding of final implementation)

  **References** (CRITICAL - Be Exhaustive):

  > The executor has NO context from your interview. References are their ONLY guide.
  > Each reference must answer: "What should I look at and WHY?"

  **Pattern References** (existing code to follow):
  - Current commenting and documentation patterns in the codebase

  **API/Type References** (contracts to implement against):
  - [DOCUMENTATION STYLE REFERENCES]

  **Test References** (testing patterns to follow):
  - [INTERNAL DOCS PATTERNS]

  **External References** (libraries and frameworks):
  - [STYLE GUIDE REFERENCES]

  **WHY Each Reference Matters** (explain the relevance):
  - Maintaining consistency with existing documentation style and structure

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY** — No human action permitted.
  > Every criterion MUST be verbable by running a command or using a tool.

  **If TDD (tests enabled):**
  - [ ] Update relevant comments and documentation sections
  - [ ] Ensure documentation matches actual implemented behavior

  **QA Scenarios (MANDATORY):**

  > **This is NOT optional. A task without QA scenarios WILL BE REJECTED.**
  >
  > Write scenario tests that verify the ACTUAL BEHAVIOR of what you built.
  > Minimum: 1 happy path + 1 failure/edge case per task.
  > Each scenario = exact tool + exact steps + exact assertions + evidence path.
  >
  > **The executing agent MUST run these scenarios after implementation.**
  > **The orchestrator WILL verify evidence files exist before marking task complete.**

  ```
  Scenario: [Code documentation accurately describes new keyword functionality]
    Tool: Documentation review
    Preconditions: All keyword verification functionality implemented
    Steps:
      1. Verify code comments explain the hybrid VAD+ASR approach
      2. Confirm comments distinguish between energy threshold and keyword verification
      3. Review documentation for configuration options 
    Expected Result: Clear documentation showing how keyword verification works
    Failure Indicators: Confusing or outdated documentation
    Evidence: .sisyphus/evidence/task-12-documentation-complete.log

  Scenario: [Troubleshooting documentation helps with keyword verification issues]
    Tool: Documentation usability test
    Steps:  
      1. Review documentation for common keyword verification problems
      2. Check clarity of error messages and configuration requirements
      3. Verify troubleshooting steps are clear and helpful
    Expected Result: Comprehensive documentation covering configuration and debugging
    Evidence: .sisyphus/evidence/task-12-troubleshooting-helpful.log
  ```

  > **Specificity requirements — every scenario MUST use:**
  > - **Selectors**: Specific CSS selectors (`.login-button`, not "the login button")
  > - **Data**: Concrete test data (`"test@example.com"`, not `"[email]"`)
  > - **Assertions**: Exact values (`text contains "Welcome back"`, not "verify it works")
  > - **Timing**: Wait conditions where relevant (`timeout: 10s`)
  > - **Negative**: At least ONE failure/edge case per task
  >
  > **Anti-patterns (your scenario is INVALID if it looks like this):**
  > - ❌ "Verify it works correctly" — HOW? What does "correctly" mean?
  > - ❌ "Check the API returns data" — WHAT data? What fields? What values?
  > - ❌ "Test the component renders" — WHERE? What selector? What content?
  > - ❌ Any scenario without an evidence path

  **Evidence to Capture:**
  - [ ] Each evidence file named: task-{N}-{scenario-slug}.{ext}
  - [ ] Updated commentary, documentation reviews

  **Commit**: YES | NO (group with final verification)
  - Message: `docs: Update for keyword verification changes`
  - Files: `inline code comments, README.md`
  - Pre-commit: `documentation verification`

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Rejection → fix → re-run.

- [ ] F1. **Design Compliance Audit** — `oracle`
  Review the full implementation against original plan. Verify keyword spot functionality is implemented - each function that compares ASR results to configured wake word is present and functional. Check no deviation from architecture in design doc. Verify energy threshold used as trigger but keyword matching required for activation. Output: `Configurable wake word [X/X verified] | Keyword verification [X/X verified] | Architecture compliance [X/X] | VERDICT`

- [ ] F2. **Performance and Resource Audit** — `unspecified-high`
  Run `top` and `cargo flamegraph` during audio processing. Monitor CPU/Memory during various activities. Verify no memory leaks during extended operation. Confirm ASR processing doesn't cause excessive resource usage. Profile audio processing loops to ensure optimal performance. Check for any new bottlenecks.
  Output: `CPU Usage [X% MAX] | Memory Stability [PASS/FAIL] | Perf Impact [-Xms] | Bottlenecks [X found] | VERDICT`

- [ ] F3. **Keyword Spot Accuracy Verification** — `unspecified-high` (+ `playwright` skill if UI involvement)
  Test actual wake word against numerous attempts and false positives. Validate detection accuracy on recordings of the actual configured wake word. Test with various speakers and pronunciations. Confirm it rejects non-wake word phrases that exceed audio threshold. Measure false positive and false negative rates. Save verification to `.sisyphus/evidence/final-accuracy-tests/`.
  Output: `Accuracy [XX%] | False Positives [X/N] | False Negatives [X/N] | Speaker Variance [X/X pass] | VERDICT`

- [ ] F4. **Configuration Consistency Check** — `deep`
  For each configurable aspect, verify it's actually respected in code. Change wake word from default and verify new value used. Test changing voice configurations while system is running and check they're picked up. Verify all fields in VoiceWakeConfig are actually used. Check backwards compatibility with older config files that lacked new fields. Flag any configuration inconsistency.
  Output: `Config Fields [X/X verified] | Dynamic Update [YES/NO] | Backwards Compatibility [PASS/FAIL] | Issues [X found] | VERDICT`

---

## Commit Strategy

- **Group 1**: `feat(voice): Implement keyword spot wake verification` — voice_wake.rs, config.rs

---

## Success Criteria

### Verification Commands
```bash
cargo check  # Expected: build successful
cargo test   # Expected: all tests pass, new keyword verification tests pass
```

### Final Checklist
- [ ] All "Must Have" present (keyword matching implemented)
- [ ] All "Must NOT Have" absent (energy threshold still used, not removed)
- [ ] New functions properly integrated
- [ ] Existing UI and window focus functionality preserved
- [ ] Configuration is actually utilized for keyword verification