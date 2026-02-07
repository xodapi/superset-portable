---
title: ClickHouse
status: public
created: 2026-01-28
tags: [базы-данных, clickhouse, аналитика, olap]
---

# ClickHouse ⚡

ClickHouse — колоночная СУБД для онлайн-аналитики (OLAP), разработанная Яндексом.

---

## Преимущества ClickHouse

- ⚡ **Скорость**: запросы в 100-1000 раз быстрее классических СУБД
- 📊 **Большие данные**: петабайты данных
- 🔄 **Репликация**: встроенная отказоустойчивость
- 💾 **Сжатие**: до 10x сжатие данных
- 🇷🇺 **Русская документация**: полная поддержка

---

## Строка подключения

### Native протокол (рекомендуется)

```
clickhouse+native://пользователь:пароль@хост:9000/база
```

### HTTP протокол

```
clickhouse+http://пользователь:пароль@хост:8123/база
```

### Примеры

**Локальный сервер:**
```
clickhouse+native://default:@localhost:9000/default
```

**С паролем:**
```
clickhouse+native://analyst:mypassword@localhost:9000/analytics
```

**Кластер (любая нода):**
```
clickhouse+native://user:pass@node1.cluster:9000/db
```

---

## Порты ClickHouse

| Порт | Протокол | Назначение |
|------|----------|------------|
| 9000 | Native | Основной (рекомендуется) |
| 8123 | HTTP | Веб-интерфейс, REST |
| 9440 | Native + TLS | Защищённый |
| 8443 | HTTPS | Защищённый веб |

---

## Особенности SQL в ClickHouse

### Движки таблиц

```sql
-- MergeTree — основной движок
CREATE TABLE sales (
    date Date,
    product String,
    amount Float64,
    quantity UInt32
) ENGINE = MergeTree()
ORDER BY (date, product);

-- ReplacingMergeTree — с дедупликацией
CREATE TABLE users (
    user_id UInt64,
    name String,
    updated_at DateTime
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY user_id;
```

### Типы данных

| ClickHouse | Описание |
|------------|----------|
| `UInt8, UInt16, UInt32, UInt64` | Беззнаковые целые |
| `Int8, Int16, Int32, Int64` | Знаковые целые |
| `Float32, Float64` | Числа с плавающей точкой |
| `String` | Строки произвольной длины |
| `Date` | Дата (YYYY-MM-DD) |
| `DateTime` | Дата и время |
| `Array(T)` | Массив типа T |
| `Nullable(T)` | Значение или NULL |

---

## Агрегатные функции

ClickHouse имеет расширенный набор функций:

```sql
-- Приблизительный подсчёт уникальных
SELECT uniqHLL12(user_id) FROM events

-- Квантили
SELECT quantiles(0.5, 0.9, 0.99)(response_time) FROM logs

-- Оконные функции
SELECT 
    date,
    revenue,
    runningAccumulate(sum(revenue)) OVER (ORDER BY date) as cumulative
FROM daily_sales

-- Массивы
SELECT arrayJoin([1, 2, 3]) as x
```

---

## Оптимизация запросов

### Используйте фильтрацию по ORDER BY

```sql
-- ✅ Хорошо: фильтр по первому столбцу ORDER BY
SELECT * FROM sales WHERE date = '2026-01-28'

-- ❌ Плохо: фильтр не по ORDER BY
SELECT * FROM sales WHERE product = 'Товар A'
```

### Избегайте SELECT *

```sql
-- ❌ Плохо
SELECT * FROM huge_table

-- ✅ Хорошо
SELECT date, product, sum(amount) FROM huge_table
GROUP BY date, product
```

### PREWHERE вместо WHERE

```sql
-- Сначала фильтрует, потом читает остальные столбцы
SELECT * FROM logs 
PREWHERE date = today()
WHERE status = 'error'
```

---

## Материализованные представления

```sql
-- Агрегация в реальном времени
CREATE MATERIALIZED VIEW mv_hourly_stats
ENGINE = SummingMergeTree()
ORDER BY (hour, endpoint)
POPULATE
AS SELECT
    toStartOfHour(timestamp) as hour,
    endpoint,
    count() as request_count,
    avg(response_time) as avg_response
FROM logs
GROUP BY hour, endpoint;
```

---

## Работа с кириллицей

ClickHouse полностью поддерживает UTF-8:

```sql
-- Работает из коробки
SELECT * FROM users WHERE name = 'Иванов Иван'

-- Функции для строк
SELECT lower('ПРИВЕТ')  -- 'привет'
SELECT upper('привет')  -- 'ПРИВЕТ'

-- Поиск подстроки
SELECT position('Привет мир', 'мир')  -- 8
```

---

## Решение проблем

### "Connection refused"

Проверьте:
1. ClickHouse запущен: `clickhouse-client --version`
2. Порт 9000 открыт
3. Привязка к нужному IP в `config.xml`

### "Authentication failed"

Проверьте `users.xml`:
```xml
<users>
    <myuser>
        <password>mypassword</password>
        <networks><ip>::/0</ip></networks>
        <profile>default</profile>
    </myuser>
</users>
```

### Запрос выполняется долго

1. Используйте `EXPLAIN` для анализа
2. Добавьте фильтрацию по ключу сортировки
3. Уменьшите количество возвращаемых строк (LIMIT)
4. Используйте сэмплирование: `SAMPLE 0.1` (10% данных)

---

## Полезные ресурсы

- [Документация ClickHouse](https://clickhouse.com/docs/ru/) (RU)
- [ClickHouse Playground](https://play.clickhouse.com/)
- [Awesome ClickHouse](https://github.com/korchasa/awesome-clickhouse)

---

## См. также

- [[Подключение базы данных]] — обзор подключений
- [[PostgreSQL]] — классическая СУБД
- [[SQL Lab]] — работа с запросами
