# hits-rs API

Бэкенд сервиса hits.

## Настройка окружения

### Запуск базы данных и Redis

Для запуска необходимых сервисов (PostgreSQL и Redis) можно использовать Docker или Podman.

**С помощью Docker:**
```bash
# Запуск PostgreSQL
docker run -d \
  --name hits \
  -e POSTGRES_DB=hits \
  -e POSTGRES_USER=lexunok \
  -e POSTGRES_PASSWORD=password \
  -p 5434:5432 \
  postgres:16

# Запуск Redis
docker run -d \
  --name redis \
  -p 6379:6379 \
  redis:latest
```

**С помощью Podman:**
```bash
# Запуск PostgreSQL
podman run -d \
  --name hits \
  -e POSTGRES_DB=hits \
  -e POSTGRES_USER=lexunok \
  -e POSTGRES_PASSWORD=password \
  -p 5434:5432 \
  docker.io/postgres:16

# Запуск Redis
podman run -d \
  --name redis \
  -p 6379:6379 \
  docker.io/redis:latest
```

## Разработка

### Миграции базы данных

Для работы с миграциями используется `sea-orm-cli`.

**Создание новой миграции:**
```bash
sea-orm-cli migrate generate <название_миграции>
```
*Пример:*
```bash
sea-orm-cli migrate generate create_user_table
```

**Применение всех миграций (накатить):**
Для применения всех ожидающих миграций, выполните:
```bash
cargo run -p migration -- up
```

**Полный сброс и применение всех миграций:**
Эта команда удалит все данные в базе данных, а затем применит все миграции с самого начала.
```bash
cargo run -p migration -- fresh
```

### Генерация сущностей (Entities)

После изменения таблиц в базе данных через миграции, необходимо обновить сущности SeaORM. Указать конкретные таблицы, чтобы избежать перезаписывания кастомных моделей. ВАЖНО!!! При генерации теряется enum Role к сожалению, тут либо подход к enum менять либо ручками возвращать ее либо не генерировать entity а от entity генерировать миграции, то есть Entity-First Workflow.
```bash
sea-orm-cli generate entity --output-dir ./entity/src --lib --entity-format dense --with-serde both --ignore-tables users invitation
```

## Ключевые изменения и решения

- Вход переносится на фронт.
- Пути теперь почти все новые, без префикса `/v1`.
- Модель приглашений теперь имеет поле `emails` (множественное число) вместо `email`, а путь для отправки - `/invitations`.
- При обработке статуса `401 Unauthorized` на фронтенде токен должен обновляться через `/refresh`.
- Регистрация сразу генерирует токены. Поля `study_group` и `telephone` опциональны.
- Возможно, для фронтенда потребуется продление срока приглашения и получение приглашения при заходе на страницу.
- Обработка `Not Found` переносится на фронтенд.
- В таблицах `invitation`, `password_change` и `email_change` поле `date_expired` переименовано в `expiry_date`.
- Таблицы `password_change` и `email_change` объединены в одну - `verification_code`.
- Обновление почты теперь требует передачи `id` модели `verification_code` из предыдущего шага.
- Добавлена пагинация в `get_users`.
- Передача ролей осуществляется строковыми значениями, например: `Admin`, `Initiator`, `TeamOwner`.
- Добавлена функциональность восстановления пользователя.
- Company_Members теперь новая таблица
- Enum SkillType возможно тоже другие
