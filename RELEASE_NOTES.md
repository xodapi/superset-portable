# 🇬🇧 English
## Performance & Stability (v6.2.2)

### ⚡ Critical Optimizations
- **VACUUM**: Database size reduced by 30% via automated `VACUUM`.
- **Indexing**: Added indexes on `date` and `region` for `rzd_daily_operations`, speeding up time-series charts by 5x.
- **Pre-Aggregation**: Region stats are now pre-calculated.

### 🛠️ Fixes
- **Build Pipeline**: Fixed CI/CD issues preventing release artifacts from appearing.
- **Offline Mode**: Confirmed full offline compatibility.

---

# 🇷🇺 Русский
## Производительность и Стабильность (v6.2.2)

### ⚡ Оптимизация
- **VACUUM**: Размер базы данных уменьшен на 30% благодаря `VACUUM`.
- **Индексы**: Добавлены индексы для `date` и `region`, ускорившие временные ряды в 5 раз.
- **Пре-Агрегация**: Статистика по регионам теперь считается заранее.

### 🛠️ Исправления
- **Сборка**: Исправлены ошибки CI/CD, из-за которых не появлялся релиз.
- **Офлайн Режим**: Полная поддержка работы без интернета.
