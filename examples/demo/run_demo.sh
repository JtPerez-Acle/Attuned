#!/bin/bash
# Run the Attuned split-screen demo

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Use the Python venv from attuned-python
VENV_PYTHON="$PROJECT_ROOT/crates/attuned-python/.venv/bin/python"

if [ ! -f "$VENV_PYTHON" ]; then
    echo "Error: Python venv not found at $VENV_PYTHON"
    echo "Run: cd crates/attuned-python && uv venv && uv pip install -e . streamlit litellm"
    exit 1
fi

# Check if streamlit is installed
if ! "$VENV_PYTHON" -c "import streamlit" 2>/dev/null; then
    echo "Installing streamlit and litellm..."
    cd "$PROJECT_ROOT/crates/attuned-python" && uv pip install streamlit litellm
fi

echo "Starting Attuned Demo..."
echo "Open http://localhost:8501 in your browser"
echo ""

"$VENV_PYTHON" -m streamlit run "$SCRIPT_DIR/streamlit_app.py" --server.headless true
