# Rust Backend

A modular Rust backend application with RESTful API endpoints, OpenAPI documentation, caching, metrics, and more.

## Features

- **RESTful API** using [Axum](https://github.com/tokio-rs/axum)
- **OpenAPI Documentation** with [Utoipa](https://github.com/juhaku/utoipa) and Swagger UI
- **Caching** with [Moka](https://github.com/moka-rs/moka)
- **Metrics Collection** using [metrics](https://github.com/metrics-rs/metrics)
- **Prometheus Integration** for metrics reporting
- **Structured Error Handling**
- **Configuration Management** with YAML files and environment variables
- **Logging** with [tracing](https://github.com/tokio-rs/tracing)
- **Comprehensive Test Suite** with ~98% code coverage

## Testing

This project has a comprehensive testing framework with various test types:

### Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run specific module tests
cargo test core::config

# Run tests with documentation examples
cargo test --doc

# Generate test coverage report (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Testing Approach

- **Unit Tests**: Located in `#[cfg(test)]` modules within implementation files
- **Integration Tests**: Located in the `/tests` directory
- **Doc Tests**: Examples in documentation that serve as tests
- **Property-Based Tests**: Using proptest to test against randomly generated inputs
- **Mock Testing**: Using mockito for HTTP requests and mock-it for components

### Testing Tools

- [tokio-test](https://docs.rs/tokio-test/latest/tokio_test/) - Testing async code
- [proptest](https://docs.rs/proptest/latest/proptest/) - Property-based testing
- [mockito](https://docs.rs/mockito/latest/mockito/) - HTTP mocking
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) - Code coverage analysis

For more details, see the [Testing Roadmap](docs/testing-roadmap.md).

## Project Structure

```
src/
├── app/                  # Application router and state
│   └── router.rs
├── cache/                # Caching functionality
│   └── cache_manager.rs
├── config/               # Configuration management
│   └── app_config.rs
├── error/                # Error handling
│   └── error_types.rs
├── handlers/             # API request handlers
│   ├── data.rs
│   ├── health.rs
│   ├── metrics.rs
│   ├── mod.rs
│   └── pet.rs
├── metrics/              # Metrics collection and reporting
│   └── metrics_service.rs
├── models/               # Data models and schemas
│   ├── mod.rs
│   └── schemas.rs
├── petstore_api/         # Generated Petstore API client
├── lib.rs                # Library module declarations
└── main.rs               # Application entry point

config/
├── default.yaml          # Default configuration
├── development.yaml      # Development environment configuration
├── production.yaml       # Production environment configuration
└── local.yaml            # Local overrides (not in version control)
```

## Getting Started

### Prerequisites

- Rust (latest stable version)
- OpenAPI Generator CLI (for API client generation)

### Installation

1. Clone the repository
2. Install dependencies:

```bash
cargo build
```

### Configuration

The application uses a layered configuration approach:

1. **YAML Configuration Files**:
   - `config/default.yaml` - Base configuration for all environments
   - `config/development.yaml` - Development-specific settings
   - `config/production.yaml` - Production-specific settings
   - `config/local.yaml` - Local overrides (not in version control)
   - `config/local-{env}.yaml` - Environment-specific local overrides

2. **Environment Variables**:
   Create a `.env` file in the project root with at minimum:

```
# Environment selection
RUN_ENV=development

# Essential environment variables
RUST_LOG=${APP_LOG_LEVEL:-info}

# Secrets (if needed)
# API_KEY=your_api_key_here
```

Environment variables can also be used to override any configuration value from the YAML files.

### Running the Server

You can run the server using the unified wrapper script:

```bash
# For development (default)
./run.sh

# For production
./run.sh --prod
```

This wrapper script will automatically choose the appropriate environment script based on the `--dev` or `--prod` flag.

#### Development Mode

For development specifically:

```bash
./run_dev.sh
```

The development script supports several options:

```bash
./run_dev.sh [OPTIONS]
```

Options:
- `--skip-gen` - Skip API model generation
- `--release` - Build and run in release mode
- `--config-dir=DIR` - Use specified config directory (default: config)
- `--env=FILE` - Use specified .env file (default: .env)
- `--environment=ENV` - Use specified environment (default: development)
- `--port=PORT` - Specify server port (default: 3000)
- `--watch` - Restart server on file changes
- `--run-migrations` - Run database migrations before starting
- `--no-health-check` - Skip health check validation after startup
- `--no-hooks` - Skip git hooks setup
- `--help` - Show help message

The script always preserves your manual settings in the API registry when generating APIs, ensuring that your customizations to `generate_api` and `generate_handlers` flags remain as you set them.

Or manually (note the run_dev.sh script has required steps to run the application so this may not work):

```bash
cargo run
```

The server will start on http://localhost:3000 by default.

### Local Database

For local development, you can use Docker to run a PostgreSQL instance:

```bash
# From the project root:
cd test/resources/docker
docker-compose -f docker-compose.dev.yml up -d
```

This will create a PostgreSQL database accessible at:
- Host: localhost
- Port: 5432
- User: postgres
- Password: postgres
- Database: app

To use with the application, ensure your `config/development.yaml` has the database section enabled:

```yaml
database:
  enabled: true
  url: "postgres://postgres:postgres@localhost:5432/app"
  max_connections: 10
  connect_timeout_seconds: 30
  idle_timeout_seconds: 300
```

> **Note**: This configuration is for local development only. Production deployments use AWS RDS.

For detailed implementation of the PostgreSQL connection, see the [PostgreSQL Integration Guide](docs/postgresql_integration.md).

## API Documentation

API documentation is available at http://localhost:3000/docs when the server is running.

## Endpoints

- `GET /health` - Health check endpoint
- `GET /metrics` - Prometheus metrics endpoint
- `GET /data` - Sample data endpoint (fetches cat facts)
- `GET /pet/{id}` - Fetch pet by ID from the Petstore API

## API Integration

This project supports easy integration with downstream APIs. To add a new API endpoint:

1. **Automated Method (Recommended)**:
   ```bash
   ./scripts/add_api.sh <api_name> <api_url> <schema_url> [endpoint_path] [param_name]
   ```

   For example:
   ```bash
   ./scripts/add_api.sh jsonplaceholder https://jsonplaceholder.typicode.com https://jsonplaceholder.typicode.com/swagger.json posts id
   ```

2. **Manual Method**:
   See the detailed guide in [API Integration Guide](docs/API_INTEGRATION.md).

## API Resource Abstraction

This project includes a powerful API resource abstraction pattern for building reliable API handlers. The pattern provides:

- **Automatic caching** of API responses to reduce latency and external API calls
- **Retry mechanism** with exponential backoff for handling transient failures
- **Consistent error handling** across all API endpoints
- **Standardized logging** for API interactions
- **Type safety** through Rust's type system

### Using the API Resource Abstraction

To use the abstraction in your handlers:

1. **Implement the `ApiResource` trait for your model**:
   ```rust
   impl ApiResource for MyModel {
       type Id = i64;  // The type of your ID field
       
       fn resource_type() -> &'static str {
           "myresource"  // Used for cache keys and logging
       }
       
       fn api_name() -> &'static str {
           "MyService"  // Used for logging
       }
   }
   ```

2. **Create a handler function using the abstraction**:
   ```rust
   pub async fn get_my_resource_handler(
       State(state): State<Arc<AppState>>,
       Path(id): Path<String>,
   ) -> Result<Json<MyModel>> {
       // Create an API handler with reliability features
       let handler = create_api_handler(
           |state, id| async move {
               // Your actual API call logic here
               // ...
           },
           ApiHandlerOptions {
               use_cache: true,
               use_retries: true,
               max_retry_attempts: 3,
               cache_ttl_seconds: 300,
               detailed_logging: true,
           },
       );
       
       handler(State(state), Path(id)).await
   }
   ```

For detailed documentation, see [API Resource Documentation](docs/api_resource.md).

## Security

### Pre-commit Hook for Sensitive Data Detection

The project includes a pre-commit hook that scans staged files for sensitive data like API keys, secrets, and database credentials to prevent accidental commits of confidential information.

The hook is automatically set up when you run `./run_dev.sh` for the first time. If you want to skip this automatic setup, use the `--no-hooks` flag:

```bash
./run_dev.sh --no-hooks
```

To manually set up the pre-commit hook:

```bash
./scripts/setup-hooks.sh
```

This will install a pre-commit hook that:
- Scans staged files for sensitive patterns like API keys, passwords, and private keys
- Blocks commits containing sensitive data
- Shows detailed information about detected sensitive data
- Can be bypassed with `git commit --no-verify` when needed

To customize the sensitive data patterns, edit `scripts/pre-commit.sh`.

### License

This project is licensed under the MIT License - see the LICENSE file for details. 