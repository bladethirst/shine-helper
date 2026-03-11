- [ ] 8. Create keyword matching utility functions

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
  - **Blocked By**: [Task 5] (needs ASR processing pattern to understand result format)

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
    Evidence: .sisyphus/evidence/task-8-utility-accurate-results.log

  Scenario: [Fuzzy matching tolerates minor pronunciation differences]
    Tool: Unit tests with variations
    Preconditions: Fuzzy matching utilities enabled
    Steps:
      1. Test with slight phonetic difference: "Shine" vs "Shiene" (transposed letters)
      2. Test with accent/tonal variation: "小 Shine" vs "小 shīne"
      3. Confirm tolerance thresholds are reasonable
    Expected Result: Minor variations match, major differences don't
    Evidence: .sisyphus/evidence/task-8-fuzzy-matching.log
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

- [ ] 9. Update error handling for ASR processing

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
  - **Blocked By**: [Task 5] (needs ASR integration first)

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
    Evidence: .sisyphus/evidence/task-9-timeout-handling.log

  Scenario: [Fallback behavior activates when ASR unavailable]
    Tool: Server unavailable simulation
    Preconditions: VOSK server stopped/cannot connect
    Steps:
      1. Configure system to use ASR-based keyword verification
      2. Stop VOSK server to make it unreachable
      3. Test audio input and observe how system handles unavailability
      4. Verify graceful degradation behavior
    Expected Result: Either notification to user or fallback mechanism
    Evidence: .sisyphus/evidence/task-9-fallback-handling.log
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

- [ ] 10. Test wake word variations and configuration changes

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
  - **Blocked By**: [Task 3] (needs wake word configuration integration)

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
  > Every criterion MUST be verbable by running a command or using a tool.

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
    Expected Result: Different wake word from config properly recognized
    Failure Indicators: Old default continues to work, new one doesn't
    Evidence: .sisyphus/evidence/task-10-chinese-chars.log

  Scenario: [Mixed-language wake words work properly]
    Tool: Mixed-language keyword test
    Steps:
      1. Configure wake word with mixed English and Chinese: "小助手Hey"
      2. Test with spoken version of mixed phrase
      3. Confirm ASR properly handles multilingual input
    Expected Result: Multilingual wake words recognized correctly
    Evidence: .sisyphus/evidence/task-10-mixed-language.log
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

- [ ] 11. Integration test with manual verification

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
  - **Blocked By**: [Tasks 1-10] (needs all components integrated)

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
    Evidence: .sisyphus/evidence/task-11-e2e-success.log

  Scenario: [Window focus and UI events work properly after end-to-end integration]
    Tool: UI behavior verification
    Preconditions: Full system running with keyword verification active
    Steps:
      1. Configure wake word and speak to trigger recognition 
      2. Verify window focus happens correctly (app window becomes active)
      3. Confirm UI state updates to listening mode properly
      4. Check all connected UI events fire as expected
    Expected Result: Complete UI flow functions as before but with keyword verification
    Evidence: .sisyphus/evidence/task-11-ui-integration.log
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

- [ ] 12. Performance verification

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
  - **Blocked By**: [Task 2] (basic implementation needed for measurement)

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
    Evidence: .sisyphus/evidence/task-12-performance-timing.log

  Scenario: [Resource usage remains reasonable during extended operation]
    Tool: Resource monitoring
    Preconditions: System running keyword verification continuously
    Steps:
      1. Monitor CPU and memory usage during keyword verification processing
      2. Run extended test (>30 mins) to look for resource leaks
      3. Monitor during rapid-fire wake attempts 
    Expected Result: Acceptable CPU and memory usage during normal operation
    Evidence: .sisyphus/evidence/task-12-resource-usage.log
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

- [ ] 13. Documentation update

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
    Evidence: .sisyphus/evidence/task-13-documentation-complete.log

  Scenario: [Troubleshooting documentation helps with keyword verification issues]
    Tool: Documentation usability test
    Steps:  
      1. Review documentation for common keyword verification problems
      2. Check clarity of error messages and configuration requirements
      3. Verify troubleshooting steps are clear and helpful
    Expected Result: Comprehensive documentation covering configuration and debugging
    Evidence: .sisyphus/evidence/task-13-troubleshooting-helpful.log
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

- **Group 1**: `feat(voice): Implemented advanced keyword spot wake verification` — voice_wake.rs, config.rs

---

## Success Criteria

### Verification Commands
```bash
cargo check  # Expected: build successful
cargo test   # Expected: all tests pass
```

### Final Checklist
- [ ] All "Must Have" present (keyword matching implemented)
- [ ] All "Must NOT Have" absent (energy threshold still used, not removed)
- [ ] All new functions properly integrated
- [ ] Existing UI and window focus functionality preserved
- [ ] Configuration is actually utilized for keyword verification
- [ ] All macro typos fixed and compilation succeeds
- [ ] Application correctly responds only to configured wake word and not false triggers
- [ ] Full integration tests pass
- [ ] Performance remains at acceptable levels