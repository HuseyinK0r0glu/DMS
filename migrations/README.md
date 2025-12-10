# Database Migrations

This directory contains SQL migration files that will be automatically executed when the PostgreSQL container is first created.

## Usage

1. Start the database:
   ```bash
   docker-compose up -d
   ```

2. The migrations in this directory will be automatically run in alphabetical order when the database is first initialized.

3. To add new migrations, create new SQL files with a numbered prefix (e.g., `02_add_new_table.sql`)

4. To reset the database (WARNING: This will delete all data):
   ```bash
   docker-compose down -v
   docker-compose up -d
   ```

## Manual Migration Execution

If you need to run migrations manually on an existing database:

```bash
# Connect to the running container
docker exec -i dms_postgres psql -U postgres -d dms < migrations/01_init.sql
```

