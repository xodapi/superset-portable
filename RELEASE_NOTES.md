# 🇬🇧 English
## 🛡️ Python Environment Fix (v6.2.6)

### 🐛 Fixed "Empty" Release
- **Issue**: Previous builds (v6.2.x) were ~4.5MB because the internal Python environment was not being populated correctly (dependencies were installing to the CI runner instead of the package).
- **Fix**: We now explicitly target the embedded Python interpreter during the build process.
- **Result**: The release size should now be correctly ~450MB, containing a fully offline-capable Superset instance.

### 📦 Quick Start
1. Unzip.
2. Run `superset-launcher.exe`.
3. Browse to `http://localhost:8088`.

---

# 🇷🇺 Русский
## 🛡️ Исправление Среды Python (v6.2.6)

### 🐛 Исправлен "Пустой" Релиз
- **Проблема**: Прошлые релизы весили ~4.5MB, так как библиотеки устанавливались не в ту папку.
- **Решение**: Теперь сборка жестко привязана к встроенному Python.
- **Результат**: Размер релиза должен быть ~450MB, все работает офлайн.

### 📦 Быстрый Старт
1. Распакуйте.
2. Запустите `superset-launcher.exe`.
3. Откройте `http://localhost:8088`.
