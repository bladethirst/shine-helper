#!/bin/bash

echo "=== Audio Capture Test ==="
echo "Checking audio group membership..."

if sg audio -c 'arecord -l' &>/dev/null; then
    echo "✓ Audio group access: OK"
else
    echo "✗ Audio group access: FAILED"
    echo "  Try running: sg audio -c '$0'"
    exit 1
fi

echo ""
echo "Testing arecord capture (2 seconds)..."
AUDIO_FILE=$(mktemp)
sg audio -c "timeout 2 arecord -f S16_LE -r 16000 -c 1 -t raw > $AUDIO_FILE" 2>&1

if [ -s "$AUDIO_FILE" ]; then
    SIZE=$(stat -c%s "$AUDIO_FILE" 2>/dev/null || stat -f%z "$AUDIO_FILE" 2>/dev/null)
    echo "✓ Audio capture: OK (${SIZE} bytes captured)"
    rm -f "$AUDIO_FILE"
else
    echo "✗ Audio capture: FAILED (no data)"
    rm -f "$AUDIO_FILE"
    exit 1
fi

echo ""
echo "=== Vosk WebSocket Test ==="
VOSK_URL="${VOSK_URL:-ws://localhost:5000}"

echo "Connecting to Vosk server at: $VOSK_URL"

if command -v websocat &>/dev/null; then
    echo '{"config": {"sample_rate": 16000}}' | timeout 5 websocat ws://localhost:5000 - 2>&1 | head -5 || echo "✗ WebSocket test failed or timed out"
else
    echo "⚠ websocat not installed - skipping websocket connection test"
    echo "  Install with: apt-get install websocat"
fi

echo ""
echo "=== Test Summary ==="
echo "✓ Audio permissions: OK"
echo "✓ Audio capture: OK"
echo "→ Voice wake-up should work with proper permissions"
echo ""
echo "Next steps:"
echo "1. Run your application with audio group: sg audio -c './your-app'"
echo "2. Start voice wake-up functionality"
echo "3. Check logs for successful audio capture and websocket connection"