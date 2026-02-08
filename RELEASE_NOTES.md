# 🇬🇧 English
## 🎯 Dashboards & Charts Fix (v6.2.7)

### 🐛 Critical Bug Fixes
- **Missing Labels**: Fixed "Data error: Missing label" in all charts. The dashboard definitions were updated to comply with the latest Superset schema.
- **Offline Maps**: Removed dependency on online Mapbox styles ("OSM key not registered"), ensuring the World Rail map loads purely from offline GeoJSON.
- **Table Sorting**: Fixed column sorting configuration in table charts.

### 📦 Quick Start
1. Unzip `superset-portable-v6.2-rzd.zip` from v6.2.7.
2. Run `superset-launcher.exe`.
3. Give it 20-30 seconds to update the dashboard database automatically on first launch.
4. Open the "RZD Analytics" or "World Railways" dashboard.

---

# 🇷🇺 Русский
## 🎯 Исправление Дашбордов (v6.2.7)

### 🐛 Исправления
- **Ошибки Графиков**: Исправлена ошибка "Missing label", из-за которой графики были пустыми. Обновлена структура JSON для совместимости с новой версией Superset.
- **Офлайн Карты**: Убрана зависимость от онлайн-стилей Mapbox, теперь карта ЖД путей работает полностью офлайн.
- **Сортировка Таблиц**: Поправлена сортировка в табличных виджетах.

### 📦 Инструкция
1. Скачайте v6.2.7.
2. Запустите `superset-launcher.exe`.
3. Подождите 20-30 секунд при первом запуске (обновится база данных).
4. Откройте дашборды.
