---
title: Настройка конфигурации
status: public
created: 2026-01-28
tags: [администрирование, конфигурация, superset_config]
---

# Настройка конфигурации ⚙️

Основные параметры в `superset_config.py`.

---

## Расположение

```
superset_home/
└── superset_config.py    ← Файл конфигурации
```

Изменения вступают в силу после перезапуска Superset.

---

## Секретный ключ

> [!CAUTION]
> Критически важная настройка для безопасности!

```python
SECRET_KEY = 'ваш-уникальный-секретный-ключ-минимум-32-символа'
```

### Генерация нового ключа

```python
import secrets
print(secrets.token_hex(32))
```

Или через Rust-лаунчер — ключ генерируется автоматически.

---

## Основные параметры

### Лимиты

```python
# Максимум строк в результате
ROW_LIMIT = 50000

# Максимум строк для выборки
SQL_MAX_ROW = 100000

# Таймаут веб-сервера (секунды)
SUPERSET_WEBSERVER_TIMEOUT = 60

# Таймаут SQL-запроса
SQLLAB_TIMEOUT = 300
```

### Кеширование

```python
# Кеш результатов (секунды)
CACHE_DEFAULT_TIMEOUT = 60 * 60 * 24  # 24 часа

# Отключить кеш
CACHE_CONFIG = {
    'CACHE_TYPE': 'NullCache',
}

# Файловый кеш
CACHE_CONFIG = {
    'CACHE_TYPE': 'FileSystemCache',
    'CACHE_DIR': 'cache',
}
```

---

## База данных метаданных

```python
# SQLite (по умолчанию)
SQLALCHEMY_DATABASE_URI = 'sqlite:///superset_home/superset.db'

# PostgreSQL
SQLALCHEMY_DATABASE_URI = 'postgresql://user:pass@localhost:5432/superset'

# MySQL
SQLALCHEMY_DATABASE_URI = 'mysql://user:pass@localhost:3306/superset'
```

---

## Языки и локализация

```python
# Доступные языки
LANGUAGES = {
    'en': {'flag': 'us', 'name': 'English'},
    'ru': {'flag': 'ru', 'name': 'Русский'},
}

# Язык по умолчанию
BABEL_DEFAULT_LOCALE = 'ru'
```

---

## CSV/Excel загрузка

```python
# Разрешённые расширения
CSV_EXTENSIONS = {'csv', 'tsv', 'txt'}
EXCEL_EXTENSIONS = {'xlsx', 'xls'}

# Максимальный размер файла
MAX_CONTENT_LENGTH = 100 * 1024 * 1024  # 100 MB

# CSV-парсинг
CSV_TO_HIVE_UPLOAD_DIRECTORY = 'tmp'
```

---

## Mapbox (карты)

```python
# Токен для карт Mapbox
MAPBOX_API_KEY = 'pk.eyJ1IjoieW91ci11c2VybmFtZSI...'
```

Получить: [mapbox.com](https://mapbox.com)

---

## Feature Flags

### Включение функций

```python
FEATURE_FLAGS = {
    # Встроенные (embedded) дашборды
    "EMBEDDED_SUPERSET": True,
    
    # Алерты и отчёты
    "ALERT_REPORTS": True,
    
    # Горизонтальный фильтр-бар
    "HORIZONTAL_FILTER_BAR": True,
    
    # Drill-by для диаграмм
    "DRILL_BY": True,
    
    # Dashboard RBAC
    "DASHBOARD_RBAC": True,
    
    # Cross-фильтры
    "DASHBOARD_CROSS_FILTERS": True,
}
```

### Отключение функций

```python
FEATURE_FLAGS = {
    # Отключить примеры
    "DISABLE_EXAMPLES": True,
}
```

---

## Безопасность

### Аутентификация

```python
from flask_appbuilder.security.manager import AUTH_DB

# Встроенная БД (по умолчанию)
AUTH_TYPE = AUTH_DB

# LDAP
# AUTH_TYPE = AUTH_LDAP

# OAuth
# AUTH_TYPE = AUTH_OAUTH
```

### Публичный доступ

```python
# Разрешить анонимный доступ
PUBLIC_ROLE_LIKE = 'Gamma'
```

### CORS

```python
ENABLE_CORS = True
CORS_OPTIONS = {
    'supports_credentials': True,
    'allow_headers': ['*'],
    'resources': ['*'],
    'origins': ['http://localhost:3000'],
}
```

### Content Security Policy

```python
TALISMAN_ENABLED = True
TALISMAN_CONFIG = {
    'content_security_policy': {
        'default-src': "'self'",
        'img-src': "'self' data:",
        'style-src': "'self' 'unsafe-inline'",
    }
}
```

---

## SMTP (отчёты на email)

```python
SMTP_HOST = 'smtp.company.ru'
SMTP_PORT = 587
SMTP_STARTTLS = True
SMTP_USER = 'superset@company.ru'
SMTP_PASSWORD = 'your_smtp_password'
SMTP_MAIL_FROM = 'superset@company.ru'

# Включить отчёты
FEATURE_FLAGS = {
    "ALERT_REPORTS": True,
}
```

---

## Логирование

```python
# Уровень логов
LOG_LEVEL = 'INFO'  # DEBUG, INFO, WARNING, ERROR

# Формат логов
LOG_FORMAT = '%(asctime)s:%(levelname)s:%(name)s:%(message)s'

# Логи в файл
import logging
from logging.handlers import RotatingFileHandler

logging.getLogger('superset').addHandler(
    RotatingFileHandler(
        'logs/superset.log',
        maxBytes=10*1024*1024,  # 10 MB
        backupCount=5
    )
)
```

---

## Celery (фоновые задачи)

Для async-запросов и алертов:

```python
from celery.schedules import crontab

class CeleryConfig:
    broker_url = 'redis://localhost:6379/0'
    result_backend = 'redis://localhost:6379/1'

CELERY_CONFIG = CeleryConfig
```

---

## Полный пример

```python
import os

# Секретный ключ
SECRET_KEY = os.environ.get('SUPERSET_SECRET_KEY', 'default-secret-key-change-me')

# База метаданных
SQLALCHEMY_DATABASE_URI = 'sqlite:///superset_home/superset.db'

# Лимиты
ROW_LIMIT = 50000
SUPERSET_WEBSERVER_TIMEOUT = 120

# Локализация
BABEL_DEFAULT_LOCALE = 'ru'
LANGUAGES = {
    'ru': {'flag': 'ru', 'name': 'Русский'},
    'en': {'flag': 'us', 'name': 'English'},
}

# Функции
FEATURE_FLAGS = {
    "DASHBOARD_CROSS_FILTERS": True,
    "EMBEDDED_SUPERSET": True,
}

# Кеш
CACHE_CONFIG = {
    'CACHE_TYPE': 'FileSystemCache',
    'CACHE_DIR': 'cache',
    'CACHE_DEFAULT_TIMEOUT': 86400,
}
```

---

## Проверка конфигурации

После изменений:

```cmd
python\python.exe -c "from superset.config import *; print('OK')"
```

---

## См. также

- [[Пользователи]] — управление учётными записями
- [[Роли]] — настройка прав
- [[Решение проблем]] — типичные ошибки
