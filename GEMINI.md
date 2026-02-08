# Gemini Project Context: hits-rs

## Project Overview

This is a backend API service named "hits-rs", written in Rust. It serves as the backend for the "hits" service.

The project is structured as a Rust workspace with multiple crates:
- `hits-rs` (root): The main binary crate that launches the service.
- `api`: The core logic of the web API. It's built using the **Axum** web framework and runs on the **Tokio** async runtime.
- `entity`: Contains the `sea-orm` entity definitions for database models.
- `migration`: Handles database migrations using `sea-orm-cli`.
- `macros`: Contains procedural macros used in the project.

The architecture follows a typical web service pattern:
- **Handlers (`api/src/handlers`):** Define the API endpoints and handle incoming requests.
- **Services (`api/src/services`):** Contain the business logic.
- **DTOs (`api/src/dtos`):** Data Transfer Objects used for request payloads and response bodies, with validation provided by the `validator` crate.
- **Database:** Uses **PostgreSQL** via the **SeaORM** async ORM.
- **Caching/Workers:** Uses **Redis** for background tasks (e.g., sending invitations).
- **Authentication:** Implemented using JSON Web Tokens (JWT).

## Building and Running

### 1. Environment Setup

The service requires a PostgreSQL database and a Redis instance. You can run them using Docker or Podman as described in the `README.md`.

**Example with Docker:**
```bash
# Start PostgreSQL
docker run -d \
  --name hits \
  -e POSTGRES_DB=hits \
  -e POSTGRES_USER=lexunok \
  -e POSTGRES_PASSWORD=password \
  -p 5434:5432 \
  postgres:16

# Start Redis
docker run -d \
  --name redis \
  -p 6379:6379 \
  redis:latest
```

### 2. Configuration

Create a `.env` file in the project root. You can use `.env.example` (if it exists) or the following template. The existing `.env` file shows these variables:
```dotenv
PORT=3000
JWT_SECRET=your_jwt_secret
DATABASE_URL="postgres://lexunok:password@localhost:5434/hits"
REDIS_URL="redis://127.0.0.1/"
CLIENT_URL="http://localhost:8080"

ADMIN_USERNAME="admin@example.com"
ADMIN_PASSWORD="strong_admin_password"

SMTP_HOST="your_smtp_host"
SMTP_USER="your_smtp_user"
SMTP_PASSWORD="your_smtp_password"
SMTP_FROM="no-reply@example.com"
```

### 3. Database Migration

Before running the application for the first time, apply the database migrations:
```bash
cargo run -p migration -- up
```

### 4. Running the Application

To build and run the project:
```bash
cargo run
```
The server will start (by default on `0.0.0.0:3000`) and you should see a debug message indicating the listening address.

### 5. Running Tests
```bash
cargo test
```

## Development Conventions

- **Database Migrations:** Use `sea-orm-cli` to create new migration files. After running migrations, regenerate the entities.
  ```bash
  # Create a new migration
  sea-orm-cli migrate generate <migration_name>

  # Apply migrations
  cargo run -p migration -- up

  # Regenerate entities from the database schema
  sea-orm-cli generate entity --output-dir ./entity/src --lib --entity-format dense --with-serde both
  ```
- **API Documentation:** The `API.md` file contains detailed documentation for all API endpoints, including request/response formats and potential errors. Keep it updated when making changes.
- **Modularity:** The project is divided into crates (`api`, `entity`, `migration`). Business logic is further separated into services, handlers, and DTOs within the `api` crate.
- **Error Handling:** The `api/src/error.rs` file defines a central `AppError` enum for consistent error handling across the application.
- **Configuration:** All configuration is managed via environment variables loaded from the `.env` file. Do not commit secrets to the repository.
