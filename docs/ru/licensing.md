# Лицензирование и публикация форка Apache Superset

## Лицензия Apache 2.0

Apache Superset распространяется под **Apache License 2.0** — одной из самых либеральных open-source лицензий.

### ✅ Что разрешено

| Действие | Разрешено |
|----------|-----------|
| Коммерческое использование | ✅ Да |
| Модификация исходного кода | ✅ Да |
| Распространение | ✅ Да |
| Создание форков | ✅ Да |
| Частное использование | ✅ Да |
| Использование патентов | ✅ Да |

### ⚠️ Обязательные условия

1. **Сохранить лицензию** — копия LICENSE файла должна быть в вашем репозитории
2. **Указать изменения** — если вы модифицировали код, нужно указать это в NOTICE или отдельном файле
3. **Сохранить copyright** — нельзя удалять существующие copyright notices

### ❌ Ограничения

- Нельзя использовать торговые марки Apache™ без разрешения
- Нельзя выдавать свой форк за официальный Apache Superset

---

## Как правильно сделать публичный форк

### Шаг 1: Создать форк на GitHub

1. Перейти на https://github.com/apache/superset
2. Нажать **Fork**
3. Выбрать ваш аккаунт/организацию

### Шаг 2: Оформить как производную работу

Создайте или обновите файл **NOTICE** в корне репозитория:

```
Superset Portable Russian Edition
Copyright 2026 [Ваше имя/организация]

This product includes software developed at
The Apache Software Foundation (http://www.apache.org/).

Modifications:
- Portable Windows version with embedded Python
- Russian localization improvements
- Local documentation in Russian
- Rust-based launcher for improved startup performance
```

### Шаг 3: Добавить свой README

Укажите в README.md:
- Что это форк Apache Superset
- Какие изменения вы внесли
- Ссылку на оригинальный проект
- Ссылку на лицензию

### Шаг 4: Выбрать название

**Рекомендации:**
- ✅ "Superset Portable" — допустимо (описательное название)
- ✅ "Портативный Superset" — допустимо
- ❌ "Apache Superset Portable" — избегать (может восприниматься как официальный)
- ❌ Использование логотипа Apache без разрешения

---

## Вклад в основной проект

### Если хотите, чтобы ваши переводы попали в официальный Superset:

1. **Подписать CLA** — Apache Individual Contributor License Agreement
   - https://www.apache.org/licenses/icla.pdf

2. **Создать Pull Request** — с вашими изменениями в переводах
   - Файл: `superset/translations/ru/LC_MESSAGES/messages.po`

3. **Следовать Contribution Guidelines**
   - https://superset.apache.org/docs/contributing/

### Процесс контрибуции переводов:

```bash
# 1. Форкнуть репозиторий
git clone https://github.com/YOUR_USERNAME/superset.git

# 2. Создать ветку
git checkout -b improve-russian-translations

# 3. Внести изменения в messages.po

# 4. Скомпилировать .mo файл
pybabel compile -d superset/translations

# 5. Создать Pull Request
```

---

## Наши модификации

### Что мы изменили в этом форке:

| Компонент | Изменение | Файлы |
|-----------|-----------|-------|
| Локализация | Добавлено 26 переводов | `messages.po` |
| Запуск | Rust-лаунчер | `src/*.rs` |
| Портативность | Embedded Python | `python/` |
| Документация | Русскоязычное руководство | `docs/ru/` |
| Конфигурация | Предустановленные настройки | `superset_home/` |

### Совместимость с upstream

Наши изменения:
- ✅ Не ломают совместимость с оригинальным Superset
- ✅ Могут быть отправлены в upstream (переводы)
- ✅ Являются дополнениями, а не заменами

---

## Контакты сообщества

- **Slack**: http://bit.ly/join-superset-slack
- **Mailing list**: dev@superset.apache.org
- **Stack Overflow**: тег `apache-superset`
- **GitHub Issues**: https://github.com/apache/superset/issues

---

*Этот документ создан для понимания правил лицензирования. Для юридических вопросов консультируйтесь со специалистом.*
