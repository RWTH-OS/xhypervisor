#!/bin/bash
# Test runner wrapper that codesigns the test binary before running it

TEST_BINARY="$1"
shift

# Codesign the test binary on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    ENTITLEMENTS="$(dirname "$0")/app.entitlements"
    if [[ -f "$ENTITLEMENTS" ]]; then
        codesign --entitlements "$ENTITLEMENTS" --force -s - "$TEST_BINARY" 2>/dev/null || true
    fi
fi

# Run the test
exec "$TEST_BINARY" "$@"
