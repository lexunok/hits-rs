# Добавить новую миграцию
```
sea-orm-cli migrate generate create_user_table
```

# Запуск бд 
```
docker run -d \
  --name hits \
  -e POSTGRES_DB=hits \
  -e POSTGRES_USER=lexunok \
  -e POSTGRES_PASSWORD=password \
  -p 5434:5432 \
  postgres:16
```
```
docker run -d \
  --name redis \
  -p 6379:6379 \
  redis:latest
```
Альтернатива на Podman
```
podman run -d \
  --name hits \
  -e POSTGRES_DB=hits \
  -e POSTGRES_USER=lexunok \
  -e POSTGRES_PASSWORD=password \
  -p 5434:5432 \
  docker.io/postgres:16
```
```
podman run -d \
  --name redis \
  -p 6379:6379 \
  docker.io/redis:latest
```

# Генерация энтити
```
sea-orm-cli generate entity --output-dir ./entity/src --lib --entity-format dense --with-serde both
```

# Сегодня будет дроп
```
cargo run -p migration -- fresh
```

Изменения:
-Вход переносим на фронт.
-Пути теперь почти все новые, ну как минимум без /v1
-Модель приглашений теперь поле emails а не email а путь на отправку /invitations.
-Не помню как было но регистрация сразу генерит токены, а study group и telephone опциональны, также возможно для фронта требуется продление срока приглашения и получение приглашения при заходе на страницу но это не точно
-Not Found на фронт переносим
-Таблица password_change теперь password_reset
-Таблица invitation и password_change поле date_expired на expiry_date