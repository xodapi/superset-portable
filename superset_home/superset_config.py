import os

# ---------------------------------------------------
# Базовые пути
# ---------------------------------------------------
_THIS_DIR = os.path.dirname(os.path.abspath(__file__))
_ROOT_DIR = os.path.dirname(_THIS_DIR)

# ---------------------------------------------------
# Безопасность
# ---------------------------------------------------
SECRET_KEY = 'dl4y6EIdA4g8cbufwxB7WL3hbSpKyClUSCYay5NNWPqvY0kYd9y1LqDm'
WTF_CSRF_ENABLED = False

# ---------------------------------------------------
# База данных метаданных Superset
# ---------------------------------------------------
SQLALCHEMY_DATABASE_URI = 'sqlite:///' + os.path.join(_THIS_DIR, 'superset.db')

# ---------------------------------------------------
# База данных с демо-данными (examples)
# ---------------------------------------------------
SQLALCHEMY_EXAMPLES_URI = 'sqlite:///' + os.path.join(_ROOT_DIR, 'examples.db')

# ---------------------------------------------------
# Отключённые функции (экономия памяти)
# ---------------------------------------------------
FEATURE_FLAGS = {
    'ALERT_REPORTS': False,
    'DASHBOARD_NATIVE_FILTERS': True,
    'DASHBOARD_CROSS_FILTERS': True,
    'DASHBOARD_NATIVE_FILTERS_SET': True,
    'ENABLE_TEMPLATE_PROCESSING': False,
    'GLOBAL_ASYNC_QUERIES': False,
    'VERSIONED_EXPORT': False,
    'THUMBNAILS': False,
    'LISTVIEWS_DEFAULT_CARD_VIEW': False,
    'ENABLE_EXPLORE_DRAG_AND_DROP': True,
    'SCHEDULED_QUERIES': False,
    'ESTIMATE_QUERY_COST': False,
    'SIP_38_VIZ_REARCHITECTURE': True,
}

# ---------------------------------------------------
# Оптимизации для слабых машин (4 ГБ RAM, Celeron)
# ---------------------------------------------------

# SQLAlchemy — ограничиваем пул соединений
SQLALCHEMY_POOL_SIZE = 2
SQLALCHEMY_MAX_OVERFLOW = 3
SQLALCHEMY_POOL_TIMEOUT = 30
SQLALCHEMY_POOL_RECYCLE = 1800

# Отключаем отслеживание модификаций (экономит RAM)
SQLALCHEMY_TRACK_MODIFICATIONS = False

# Лимиты запросов
ROW_LIMIT = 5000
SQL_MAX_ROW = 10000
SQLLAB_TIMEOUT = 60
SUPERSET_WEBSERVER_TIMEOUT = 120

# Количество воркеров — минимум для слабых CPU
SUPERSET_WORKERS = 1
SUPERSET_WEBSERVER_THREADS = 2

# ---------------------------------------------------
# Кэш — простой in-memory (без Redis/Memcached)
# ---------------------------------------------------
CACHE_CONFIG = {
    'CACHE_TYPE': 'SimpleCache',
    'CACHE_DEFAULT_TIMEOUT': 600,
    'CACHE_THRESHOLD': 100,
}

DATA_CACHE_CONFIG = {
    'CACHE_TYPE': 'SimpleCache',
    'CACHE_DEFAULT_TIMEOUT': 600,
    'CACHE_THRESHOLD': 50,
}

FILTER_STATE_CACHE_CONFIG = {
    'CACHE_TYPE': 'SimpleCache',
    'CACHE_DEFAULT_TIMEOUT': 600,
    'CACHE_THRESHOLD': 30,
}

EXPLORE_FORM_DATA_CACHE_CONFIG = {
    'CACHE_TYPE': 'SimpleCache',
    'CACHE_DEFAULT_TIMEOUT': 600,
    'CACHE_THRESHOLD': 30,
}

# ---------------------------------------------------
# Отключаем Celery (не нужен для портативной версии)
# ---------------------------------------------------


class CeleryConfig:
    pass


CELERY_CONFIG = CeleryConfig

# ---------------------------------------------------
# Локализация / Language settings
# ---------------------------------------------------
BABEL_DEFAULT_LOCALE = "ru"
BABEL_DEFAULT_FOLDER = "superset/translations"

LANGUAGES = {
    "ru": {"flag": "ru", "name": "Русский"},
    "en": {"flag": "us", "name": "English"},
}

# ---------------------------------------------------
# Документация (локальная)
# ---------------------------------------------------
DOCUMENTATION_URL = "http://localhost:8089"
DOCUMENTATION_TEXT = "Документация"
DOCUMENTATION_ICON = "book"

# ---------------------------------------------------
# Карты (оффлайн — отключаем внешние запросы)
# ---------------------------------------------------
MAPBOX_API_KEY = ""

# ---------------------------------------------------
# Безопасность загрузки файлов
# ---------------------------------------------------
ALLOWED_EXTENSIONS = {'csv', 'tsv', 'xls', 'xlsx'}
CSV_EXTENSIONS = {'csv', 'tsv'}
EXCEL_EXTENSIONS = {'xls', 'xlsx'}
UPLOAD_FOLDER = os.path.join(_THIS_DIR, 'uploads')

# ---------------------------------------------------
# Логирование — минимальное
# ---------------------------------------------------
LOG_FORMAT = '%(asctime)s:%(levelname)s:%(name)s:%(message)s'
LOG_LEVEL = 'WARNING'
ENABLE_TIME_ROTATE = False

# ---------------------------------------------------
# Тема по умолчанию
# ---------------------------------------------------
APP_NAME = "РЖД Аналитика"
APP_ICON = ""

# Стартовая страница — сразу на дашборд
# DEFAULT_LANDING_PAGE = "/superset/dashboard/rzd_analytics/"

# ---------------------------------------------------
# Прочее
# ---------------------------------------------------
# Отключаем проверку обновлений и телеметрию
ENABLE_PROXY_FIX = False
PREVENT_UNSAFE_DB_CONNECTIONS = False

# Отключаем Content Security Policy для локальной работы
TALISMAN_ENABLED = False
CONTENT_SECURITY_POLICY_WARNING = False

# Debug mode — выключен для экономии памяти
DEBUG = False
FLASK_USE_RELOAD = False
