# Roz API Project

## Tech Stack
- Axum web framework
- Tokio async runtime
- Serde for JSON serialization

## Development
- Server runs on http://localhost:3000
- Health check endpoint available at GET /health

## Project Structure
- `src/main.rs` - Application entry point and route definitions
- Additional routes and handlers should be organized into modules as the project grows

## Best Practices
- Use async/await for handling requests
- Implement proper error handling using Result types
- Add logging for all endpoints
- Document all API endpoints
