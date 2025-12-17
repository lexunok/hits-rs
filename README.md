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
-При 401 обработке на фронте токен должен обновляться /refresh
-Не помню как было но регистрация сразу генерит токены, а study group и telephone опциональны, также возможно для фронта требуется продление срока приглашения и получение приглашения при заходе на страницу но это не точно
-Not Found на фронт переносим
-Таблица invitation и password_change и email_change поле date_expired на expiry_date
-Таблица password_change объедениена с email_change и теперь verification_code
-Обновление почты теперь требует передачи id модельки verification_code из прошлого шага
-Добавил пагинацию в get_users
-Передавать enum как Admin, Initiator, TeamOwner и тд