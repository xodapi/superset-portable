---
title: Jinja-шаблоны
status: public
created: 2026-01-28
tags: [sql, jinja, шаблоны, параметры]
---

# Jinja-шаблоны 🔧

Динамические параметры в SQL-запросах.

---

## Что такое Jinja

**Jinja2** — язык шаблонов Python. В Superset используется для:
- Динамических фильтров
- Переменных от пользователя
- Условной логики
- Макросов

---

## Синтаксис

| Синтаксис | Назначение |
|-----------|------------|
| `{{ ... }}` | Вывод значения |
| `{% ... %}` | Управляющие конструкции |
| `{# ... #}` | Комментарии |

---

## Встроенные переменные

### Временные переменные

| Переменная | Описание |
|------------|----------|
| `{{ from_dttm }}` | Начало временного диапазона |
| `{{ to_dttm }}` | Конец временного диапазона |
| `{{ time_column }}` | Столбец времени датасета |
| `{{ time_grain }}` | Гранулярность (day, week...) |

### Пользователь

| Переменная | Описание |
|------------|----------|
| `{{ current_user_id() }}` | ID пользователя |
| `{{ current_username() }}` | Имя пользователя |
| `{{ current_user_email() }}` | Email пользователя |

### Кеширование

| Переменная | Описание |
|------------|----------|
| `{{ cache_key_wrapper("key") }}` | Уникальный ключ кеша |

---

## Примеры

### Фильтрация по времени

```sql
SELECT 
    date,
    SUM(amount) as revenue
FROM sales
WHERE date BETWEEN '{{ from_dttm }}' AND '{{ to_dttm }}'
GROUP BY date
```

### Фильтрация по пользователю

```sql
SELECT *
FROM projects
WHERE owner = '{{ current_username() }}'
```

### Row Level Security

```sql
SELECT *
FROM sensitive_data
WHERE department = '{{ current_user().department }}'
```

---

## Фильтры дашборда

### filter_values()

Получение значений фильтра:

```sql
SELECT *
FROM orders
WHERE region IN {{ filter_values('region') | where_in }}
```

### Проверка наличия фильтра

```sql
SELECT *
FROM orders
{% if filter_values('region') %}
WHERE region IN {{ filter_values('region') | where_in }}
{% endif %}
```

### Несколько фильтров

```sql
SELECT *
FROM orders
WHERE 1=1
{% if filter_values('region') %}
  AND region IN {{ filter_values('region') | where_in }}
{% endif %}
{% if filter_values('status') %}
  AND status IN {{ filter_values('status') | where_in }}
{% endif %}
```

---

## Условная логика

### IF/ELSE

```sql
SELECT 
    {% if time_grain == 'day' %}
        date,
    {% elif time_grain == 'week' %}
        date_trunc('week', date) as date,
    {% else %}
        date_trunc('month', date) as date,
    {% endif %}
    SUM(amount) as total
FROM sales
GROUP BY 1
```

### Тернарный оператор

```sql
SELECT *
FROM sales
ORDER BY {{ 'amount DESC' if sort_by_amount else 'date DESC' }}
```

---

## Циклы

### FOR

```sql
SELECT 
    {% for col in ['region', 'product', 'category'] %}
        {{ col }},
    {% endfor %}
    SUM(amount) as total
FROM sales
GROUP BY 
    {% for col in ['region', 'product', 'category'] %}
        {{ col }}{{ ',' if not loop.last }}
    {% endfor %}
```

---

## Фильтры Jinja

### Встроенные фильтры

| Фильтр | Описание | Пример |
|--------|----------|--------|
| `lower` | В нижний регистр | `{{ name \| lower }}` |
| `upper` | В верхний регистр | `{{ name \| upper }}` |
| `default` | Значение по умолчанию | `{{ val \| default('N/A') }}` |
| `join` | Объединить список | `{{ items \| join(', ') }}` |

### Специальные фильтры Superset

| Фильтр | Описание |
|--------|----------|
| `where_in` | Форматирует для IN ('a', 'b') |

### Пример where_in

Входные данные: `['Moscow', 'SPb']`

```sql
-- До фильтра
WHERE region IN {{ filter_values('region') }}
-- Результат: WHERE region IN ['Moscow', 'SPb']  -- Ошибка!

-- С фильтром
WHERE region IN {{ filter_values('region') | where_in }}
-- Результат: WHERE region IN ('Moscow', 'SPb')  -- Правильно!
```

---

## Макросы

### Определение макроса

```sql
{% macro date_filter(col) %}
    {{ col }} BETWEEN '{{ from_dttm }}' AND '{{ to_dttm }}'
{% endmacro %}

SELECT *
FROM sales
WHERE {{ date_filter('sale_date') }}
```

### Использование

Макросы помогают не дублировать код.

---

## Безопасность

### SQL-инъекции

> [!WARNING]
> Jinja-шаблоны выполняются ДО SQL — будьте осторожны!

❌ **Опасно:**
```sql
SELECT * FROM {{ table_name }}  -- Если table_name от пользователя!
```

✅ **Безопасно:**
```sql
SELECT * FROM sales
WHERE region = '{{ filter_values("region")[0] | e }}'  -- | e — экранирование
```

### Whitelist подход

Разрешайте только известные значения:

```sql
{% set allowed = ['sales', 'orders', 'users'] %}
{% if table_name in allowed %}
    SELECT * FROM {{ table_name }}
{% else %}
    SELECT 'Invalid table' as error
{% endif %}
```

---

## Отладка

### Просмотр результата

В SQL Lab есть кнопка **"Show Jinja template"** для просмотра результата после обработки шаблона.

### Логирование

```sql
{# Отладочный комментарий: {{ from_dttm }} #}
SELECT *
FROM sales
WHERE date >= '{{ from_dttm }}'
```

---

## Решение проблем

### "Undefined variable"

Переменная не определена. Используйте `default`:

```sql
{{ my_var | default('default_value') }}
```

### Неправильное экранирование

Проверьте, что строки в кавычках:

```sql
WHERE name = '{{ name }}'  -- Кавычки снаружи
```

### Пустой filter_values

Проверяйте наличие:

```sql
{% if filter_values('region') %}
    AND region IN {{ filter_values('region') | where_in }}
{% endif %}
```

---

## См. также

- [[SQL Lab]] — редактор SQL
- [[Виртуальные датасеты]] — датасеты с параметрами
- [[Фильтры]] — фильтры дашбордов
