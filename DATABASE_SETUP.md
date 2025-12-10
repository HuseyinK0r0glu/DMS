# Database Setup Guide

This guide will help you set up a PostgreSQL database using Docker for the DMS (Document Management System) project.

## Prerequisites

- Docker and Docker Compose installed on your system
- Your `.env` file should contain:
  ```
  DATABASE_URL=postgres://postgres:newpassword@localhost:5432/dms
  ```

## Quick Start

Simply run:

```bash
docker-compose up -d
```

That's it! This will:
- Start the PostgreSQL container
- Create the `dms` database automatically
- Run all SQL files in the `migrations/` folder automatically (creates all tables)

The migrations run automatically because the `migrations/` folder is mounted to `/docker-entrypoint-initdb.d` in the PostgreSQL container, which executes all SQL files on first startup.

### Verify Everything Works

1. **Check the container is running:**
   ```bash
   docker-compose ps
   ```

2. **View the logs to see migrations running:**
   ```bash
   docker-compose logs postgres
   ```

3. **Connect to the database:**
   ```bash
   docker exec -it dms_postgres psql -U postgres -d dms
   ```

## Database Connection

The database will be available at:
- **Host:** localhost
- **Port:** 5432
- **Database:** dms
- **Username:** postgres
- **Password:** newpassword

## Migrations

### Automatic Migrations

When you first start the container, all SQL files in the `migrations/` directory will be automatically executed in alphabetical order.

### Manual Migration Execution

If you need to run a migration file manually (e.g., after adding a new one):

```bash
docker exec -i dms_postgres psql -U postgres -d dms < migrations/01_init.sql
```

### Creating New Migrations

1. Create a new SQL file in the `migrations/` directory with a numbered prefix:
   ```
   migrations/02_add_new_table.sql
   migrations/03_add_indexes.sql
   ```

2. The migrations will run automatically on fresh database setups, or you can run them manually using the script above.

## Connecting to the Database

### Using psql (via Docker)

```bash
docker exec -it dms_postgres psql -U postgres -d dms
```

### Using psql (local installation)

```bash
psql postgres://postgres:newpassword@localhost:5432/dms
```

### Using a GUI Tool

Use any PostgreSQL client (like pgAdmin, DBeaver, or TablePlus) with the connection details above.

## Database Management

### Stop the Database

```bash
docker-compose down
```

### Stop and Remove All Data (WARNING: This deletes everything)

```bash
docker-compose down -v
```

### Restart the Database

```bash
docker-compose restart
```

### View Database Logs

```bash
docker-compose logs -f postgres
```

## Current Tables

The initial migration (`01_init.sql`) creates the following tables:

1. **documents** - Logical document records (metadata only)
2. **document_versions** - Physical file versions with versioning support
3. **document_metadata** - Dynamic key-value metadata pairs for documents

### Schema Design

The schema uses a normalized design that separates:
- **Logical documents** (`documents` table) - Represents a document concept with title and category
- **Physical files** (`document_versions` table) - Stores actual file data with versioning (v1, v2, v3...)
- **Metadata** (`document_metadata` table) - Flexible key-value pairs for additional document properties

### Key Features

- **Versioning**: Each document can have multiple versions tracked in `document_versions`
- **Data Integrity**: File checksums (MD5 or SHA-256) for integrity verification
- **Unique Constraints**: Prevents duplicate version numbers per document
- **Cascading Deletes**: Deleting a document automatically removes its versions and metadata
- **Automatic Timestamps**: `created_at` and `updated_at` fields with automatic triggers
- **Simplified Access**: No user authentication required - all users can upload files

## Troubleshooting

### Port Already in Use

If port 5432 is already in use, you can change it in `docker-compose.yml`:

```yaml
ports:
  - "5433:5432"  # Change 5433 to any available port
```

Then update your `.env` file accordingly.

### Database Not Ready

Wait a few seconds after starting the container. You can check if it's ready with:

```bash
docker exec dms_postgres pg_isready -U postgres
```

### Reset Everything

To completely reset the database:

```bash
docker-compose down -v
docker-compose up -d
```

This will remove all data and re-run all migrations.

