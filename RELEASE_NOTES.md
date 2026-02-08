# 🇬🇧 English
## Offline Fixes & Performance (v6.2.1)

### 🚨 Critical Fixes
- **Fixed "Empty Release"**: The previous build was missing the Python engine. This release is a full offline package (~450MB).
- **Fixed Offline Maps**: Maps now load correctly even without the launcher ensuring "Global Networks" works 100% offline.

### ⚡ Performance
- **Pre-Aggregation**: Added `rzd_region_agg` to speed up dashboard queries on weak hardware.
- **Optimized Metadata**: Improved dashboard Loading times.

---

# 🇷🇺 Русский
## Офлайн Фиксы и Производительность (v6.2.1)

### 🚨 Критические Исправления
- **Исправлен "Пустой Релиз"**: Прошлый билд не включал Python. Этот релиз - полный офлайн пакет (~450MB).
- **Исправлены Офлайн Карты**: Карты теперь грузятся корректно даже без лаунчера.

### ⚡ Производительность
- **Пре-Агрегация**: Добавлена таблица `rzd_region_agg` для ускорения дашбордов на слабых ПК.
- **Оптимизация Метаданных**: Ускорено открытие дашбордов.
