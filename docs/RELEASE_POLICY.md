# Release Policy / Политика Релизов

Starting from v6.2, all releases must strictly follow this policy.
Начиная с версии v6.2, все релизы должны строго следовать этой политике.

## 1. Bilingual Description / Двуязычное Описание

Every release MUST have a description in **both English and Russian**.
Каждый релиз ОБЯЗАН иметь описание на **английском и русском языках**.

### Format / Формат:

```markdown
# 🇬🇧 English
## New Features
- Feature A
- Feature B

## Fixes
- Fixed bug X

---

# 🇷🇺 Русский
## Новые Возможности
- Функция А
- Функция Б

## Исправления
- Исправлена ошибка X
```

## 2. Release Process / Процесс Релиза

1.  **Create Notes**: Create/Update `RELEASE_NOTES.md` in the root directory with the description above.
    **Создать Заметки**: Создайте или обновите файл `RELEASE_NOTES.md` в корне, заполнив его по шаблону выше.

2.  **Commit**: Commit the notes and code changes.
    **Закоммитить**: Закоммитьте изменения кода и заметок.

3.  **Tag**: Create a signed tag.
    **Тег**: Создайте подписанный тег.
    ```bash
    git tag v6.X
    git push origin v6.X
    ```

4.  **Automation**: GitHub Actions will automatically read `RELEASE_NOTES.md` and attach it to the release.
    **Автоматизация**: GitHub Actions автоматически прочитает `RELEASE_NOTES.md` и добавит его в релиз.
