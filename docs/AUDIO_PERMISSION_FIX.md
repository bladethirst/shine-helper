# Voice Wake-up Audio Permission Fix

## Problem
When using voice wake-up functionality, the websocket connection to Vosk server fails with:
```
websockets.exceptions.ConnectionClosedError: code = 1006 (connection closed abnormally [internal])
asyncio.exceptions.IncompleteReadError: 0 bytes read on a total of 2 expected bytes
```

## Root Cause
The application uses `arecord` to capture audio, but the user running the application does not have permission to access audio devices. The `arecord` command fails silently, causing no audio data to be sent to the Vosk server, which then times out and closes the connection.

## Fix

### Option 1: Add User to Audio Group (Recommended)
```bash
# Add current user to audio group
usermod -aG audio $USER

# Log out and log back in for changes to take effect
```

### Option 2: Run with Audio Group (Immediate)
```bash
# Run application with audio group permissions
sg audio -c "./your-application"
```

### Option 3: Use Test Script
```bash
# Verify audio permissions work
bash test_audio.sh

# Or run with explicit audio group
sg audio -c 'bash test_audio.sh'
```

## Verification

1. Test audio capture:
   ```bash
   sg audio -c 'arecord -l'
   ```

2. Run the test script:
   ```bash
   bash test_audio.sh
   ```

3. Start your application with audio group permissions and check logs for successful audio capture.

## Code Improvements

The following improvements were made to help diagnose this issue in the future:

1. **Better error messages** - Added helpful hint about audio group permissions when `arecord` fails
2. **Stderr logging** - Added stderr capture to see `arecord` error messages
3. **Test script** - Created `test_audio.sh` to verify audio capture before running the application

## Files Modified

- `src-tauri/src/voice/recognition.rs` - Improved error handling and logging
- `test_audio.sh` - New test script for audio verification