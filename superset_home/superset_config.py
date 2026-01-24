import os 
SECRET_KEY = 'portable-superset-secret-key-change-me' 
SQLALCHEMY_DATABASE_URI = 'sqlite:///' + os.path.join(os.path.dirname(__file__), 'superset.db') 
WTF_CSRF_ENABLED = False 
FEATURE_FLAGS = {'ALERT_REPORTS': False} 
CACHE_CONFIG = {'CACHE_TYPE': 'SimpleCache', 'CACHE_DEFAULT_TIMEOUT': 300}

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
 
