import os 
SECRET_KEY = 'portable-superset-secret-key-change-me' 
SQLALCHEMY_DATABASE_URI = 'sqlite:///' + os.path.join(os.path.dirname(__file__), 'superset.db') 
WTF_CSRF_ENABLED = False 
FEATURE_FLAGS = {'ALERT_REPORTS': False} 
CACHE_CONFIG = {'CACHE_TYPE': 'SimpleCache', 'CACHE_DEFAULT_TIMEOUT': 300}

# ---------------------------------------------------
# Examples Database (для примеров дашбордов)
# ---------------------------------------------------
# Путь к базе данных с примерами
SQLALCHEMY_EXAMPLES_URI = 'sqlite:///' + os.path.join(os.path.dirname(os.path.dirname(__file__)), 'examples.db')


# ---------------------------------------------------
# Локализация / Language settings
# ---------------------------------------------------
# Русский язык по умолчанию
BABEL_DEFAULT_LOCALE = "ru"

# Доступные языки (переключатель появится в правом верхнем углу)
LANGUAGES = {
    "ru": {"flag": "ru", "name": "Русский"},
    "en": {"flag": "us", "name": "English"},
}

# ---------------------------------------------------
# Документация / Documentation
# ---------------------------------------------------
# Ссылка на локальную документацию (откроется в новой вкладке)
DOCUMENTATION_URL = "http://localhost:8089"
DOCUMENTATION_TEXT = "Документация"
DOCUMENTATION_ICON = "book"

# ---------------------------------------------------
# Карты / Maps (для закрытого контура)
# ---------------------------------------------------
# Для работы карт в закрытом контуре нужен локальный тайл-сервер
# Пример: OpenMapTiles, TileServer GL
# По умолчанию отключаем внешние запросы (OSM не будет работать без интернета)
# 
# Если у вас есть локальный тайл-сервер, раскомментируйте и настройте:
# MAPBOX_API_KEY = ""
# DECKGL_BASE_MAP = [
#     ['tile://http://localhost:8080/styles/osm-bright/{z}/{x}/{y}.png', 'Local OSM'],
# ]

# Отключаем mapbox чтобы не было ошибок без интернета
MAPBOX_API_KEY = ""

 
