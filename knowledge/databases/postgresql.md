---
title: PostgreSQL
status: public
created: 2026-01-28
tags: [базы-данных, postgresql, сервер]
---

# PostgreSQL 🐘

PostgreSQL — мощная реляционная СУБД с открытым исходным кодом.

---

## Строка подключения

### Базовый формат

```
postgresql://пользователь:пароль@хост:порт/база
```

### Примеры

**Локальный сервер:**
```
postgresql://postgres:mypassword@localhost:5432/analytics
```

**Удалённый сервер:**
```
postgresql://analyst:secret@192.168.1.100:5432/warehouse
```

**С SSL:**
```
postgresql://user:pass@host:5432/db?sslmode=require
```

---

## Параметры SSL

| Режим | Описание |
|-------|----------|
| `disable` | Без SSL |
| `allow` | SSL если сервер требует |
| `prefer` | SSL если возможно (по умолчанию) |
| `require` | Обязательный SSL |
| `verify-ca` | SSL + проверка CA |
| `verify-full` | SSL + проверка CA + хоста |

---

## Создание пользователя для Superset

Рекомендуется создавать отдельного пользователя только для чтения:

```sql
-- Создать пользователя
CREATE USER superset_reader WITH PASSWORD 'strong_password';

-- Права на чтение схемы public
GRANT CONNECT ON DATABASE analytics TO superset_reader;
GRANT USAGE ON SCHEMA public TO superset_reader;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO superset_reader;

-- Автоматические права на новые таблицы
ALTER DEFAULT PRIVILEGES IN SCHEMA public 
GRANT SELECT ON TABLES TO superset_reader;
```

> [!IMPORTANT]
> Не давайте права DELETE, UPDATE, DROP пользователю для аналитики!

---

## Работа с кириллицей

### Кодировка базы данных

При создании базы укажите UTF-8:

```sql
CREATE DATABASE mydb 
WITH ENCODING 'UTF8' 
LC_COLLATE='ru_RU.UTF-8' 
LC_CTYPE='ru_RU.UTF-8';
```

### Проверка кодировки

```sql
SHOW client_encoding;
SHOW server_encoding;
```

### Установка кодировки в сессии

```sql
SET client_encoding TO 'UTF8';
```

---

## Схемы (Schemas)

PostgreSQL поддерживает схемы для организации таблиц:

```
postgresql://user:pass@host:5432/db?options=-csearch_path%3Dmy_schema
```

Или настройте в Superset:
1. Подключите базу
2. В настройках укажите **"Schema"** = `my_schema`

---

## Оптимизация запросов

### Индексы

```sql
-- Обычный индекс
CREATE INDEX idx_orders_date ON orders(order_date);

-- Составной индекс
CREATE INDEX idx_orders_date_status ON orders(order_date, status);

-- Частичный индекс
CREATE INDEX idx_active_users ON users(email) WHERE is_active = true;
```

### EXPLAIN ANALYZE

```sql
EXPLAIN ANALYZE 
SELECT * FROM orders 
WHERE order_date > '2026-01-01' 
LIMIT 1000;
```

### Статистика

```sql
-- Обновить статистику
ANALYZE orders;

-- Подробная статистика
ANALYZE VERBOSE orders;
```

---

## Материализованные представления

Для ускорения сложных запросов:

```sql
-- Создание
CREATE MATERIALIZED VIEW mv_daily_sales AS
SELECT 
    date_trunc('day', order_date) as day,
    sum(amount) as total_sales,
    count(*) as order_count
FROM orders
GROUP BY 1;

-- Обновление
REFRESH MATERIALIZED VIEW mv_daily_sales;

-- Автообновление (через pg_cron)
SELECT cron.schedule('refresh_sales', '0 * * * *', 
    'REFRESH MATERIALIZED VIEW mv_daily_sales');
```

---

## Расширения PostgreSQL

Полезные расширения для аналитики:

| Расширение | Назначение |
|------------|------------|
| `pg_stat_statements` | Статистика запросов |
| `pg_trgm` | Нечёткий поиск |
| `hstore` | Key-value хранение |
| `tablefunc` | Crosstab / сводные таблицы |
| `timescaledb` | Временные ряды |

---

## Решение проблем

### "Connection refused"

Проверьте:
1. PostgreSQL запущен: `pg_isready -h localhost`
2. Разрешены подключения в `pg_hba.conf`
3. Firewall открыт для порта 5432

### "password authentication failed"

1. Проверьте правильность пароля
2. Проверьте метод аутентификации в `pg_hba.conf`

### "permission denied for table"

```sql
GRANT SELECT ON TABLE tablename TO superset_reader;
```

### Медленные запросы

1. Проверьте план выполнения: `EXPLAIN ANALYZE ...`
2. Добавьте индексы
3. Обновите статистику: `ANALYZE tablename`
4. Увеличьте `work_mem` для сложных сортировок

---

## См. также

- [[Подключение базы данных]] — обзор подключений
- [[ClickHouse]] — для больших объёмов данных
- [[SQL Lab]] — работа с запросами
