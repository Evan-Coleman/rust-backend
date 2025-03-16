#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

# Parse command line arguments
SKIP_GEN=false
RELEASE_MODE=false
CONFIG_DIR="config"
ENV_FILE=".env"
RUN_ENV="development"

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --skip-gen           Skip API model generation"
    echo "  --release            Build and run in release mode"
    echo "  --config-dir=DIR     Use specified config directory (default: config)"
    echo "  --env=FILE           Use specified .env file (default: .env)"
    echo "  --environment=ENV    Use specified environment (default: development)"
    echo "  --help               Show this help message"
}

for arg in "$@"; do
    case $arg in
        --skip-gen)
            SKIP_GEN=true
            shift
            ;;
        --release)
            RELEASE_MODE=true
            shift
            ;;
        --config-dir=*)
            CONFIG_DIR="${arg#*=}"
            shift
            ;;
        --env=*)
            ENV_FILE="${arg#*=}"
            shift
            ;;
        --environment=*)
            RUN_ENV="${arg#*=}"
            shift
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            print_usage
            exit 1
            ;;
    esac
done

# Header
echo "==================================================="
echo "  Petstore API Server with Utoipa Integration"
echo "==================================================="

# Check for required tools
echo "Checking dependencies..."
if ! command -v openapi-generator &> /dev/null; then
    echo "Warning: OpenAPI Generator is not installed."
    echo "This is needed for API generation. You can install it from: https://openapi-generator.tech/docs/installation/"
    echo "Continuing without API generation capabilities..."
fi

# Check if config files exist
if [ -d "$CONFIG_DIR" ]; then
    echo "Using config directory: $CONFIG_DIR"
    
    # Check for environment-specific config file
    if [ -f "$CONFIG_DIR/$RUN_ENV.yaml" ]; then
        echo "Found environment config: $CONFIG_DIR/$RUN_ENV.yaml"
    elif [ -f "$CONFIG_DIR/default.yaml" ]; then
        echo "Found default config: $CONFIG_DIR/default.yaml"
    else
        echo "Warning: No configuration files found in $CONFIG_DIR. Using defaults."
    fi
    
    # Export CONFIG_DIR for the application
    export CONFIG_DIR="$CONFIG_DIR"
    # Export RUN_ENV for the application
    export RUN_ENV="$RUN_ENV"
else
    echo "Warning: Config directory $CONFIG_DIR not found. Using defaults."
fi

if [ -f "$ENV_FILE" ]; then
    echo "Using environment file: $ENV_FILE"
    # Load environment variables
    export $(grep -v '^#' "$ENV_FILE" | xargs)
else
    echo "Warning: Environment file $ENV_FILE not found. Using defaults."
fi

# Skip API generation step as we now use the scripts/add_api.sh script on demand
if [ "$SKIP_GEN" = true ]; then
    echo "Skipping API model generation (--skip-gen flag used)"
    export SKIP_API_GEN=1
else
    echo "Note: API generation is now handled by scripts/add_api.sh"
    echo "Run this script separately to add new APIs"
fi

# Build the project
echo "Building the project (this will run the build script to add Utoipa annotations)..."
if [ "$RELEASE_MODE" = true ]; then
    echo "Building in release mode..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo "Error: Release build failed. See errors above."
        exit 1
    fi
    EXEC_PATH="./target/release/rust-backend"
else
    cargo build
    if [ $? -ne 0 ]; then
        echo "Error: Debug build failed. See errors above."
        exit 1
    fi
    EXEC_PATH="./target/debug/rust-backend"
fi

echo "Build successful. Starting server..."

# Set RUST_LOG if not already set
if [ -z "$RUST_LOG" ]; then
    export RUST_LOG=info
    echo "Setting log level to info (RUST_LOG=info)"
fi

# Run the executable
echo "Starting server..."
echo "Press Ctrl+C to stop the server."
echo "---------------------------------------------------"
"$EXEC_PATH"

# This part will execute after server shutdown (Ctrl+C)
echo "Server stopped."
