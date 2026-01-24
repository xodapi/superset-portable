# Демо-данные РЖД для Superset

## Файлы данных

| Файл | Описание | Записей |
|------|----------|---------|
| `rzd_stations.csv` | Крупнейшие ж/д станции России | 50 |
| `rzd_monthly_stats.csv` | Статистика перевозок по месяцам | 13 |
| `rzd_cargo_types.csv` | Распределение грузов по типам | 10 |
| `rzd_stations.geojson` | GeoJSON для карт | 15 |

## Импорт в Superset

### Шаг 1: Загрузка CSV
1. Откройте Superset → **Данные** → **Загрузить CSV в базу данных**
2. Выберите файл `rzd_stations.csv`
3. Укажите имя таблицы: `rzd_stations`
4. Нажмите **Создать**
5. Повторите для `rzd_monthly_stats.csv` и `rzd_cargo_types.csv`

### Шаг 2: Создание датасета
1. Перейдите **Данные** → **Датасеты**
2. Нажмите **+ Датасет**
3. Выберите базу данных и таблицу `rzd_stations`

### Шаг 3: Создание диаграмм

**ТОП-10 станций по пассажиропотоку:**
- Тип: Bar Chart
- Метрика: SUM(passengers_day)
- Группировка: name
- Сортировка: по убыванию

**Распределение грузов:**
- Тип: Pie Chart
- Метрика: SUM(volume_mln_tons)
- Группировка: cargo_type

**Динамика перевозок:**
- Тип: Line Chart
- Метрика: passengers_mln, cargo_mln_tons
- Временное измерение: month/year

### Шаг 4: Карта станций
Для отображения станций на карте нужен Mapbox API ключ.
В офлайн-режиме используйте таблицу с координатами.

## Расположение файлов
```
docs/demo_data/
├── rzd_stations.csv
├── rzd_monthly_stats.csv
├── rzd_cargo_types.csv
├── rzd_stations.geojson
└── README.md
```
