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

# Генерация энтити
```
sea-orm-cli generate entity --output-dir ./entity/src --lib --entity-format dense --with-serde both
```

# Сегодня будет дроп
```
cargo run -p migration -- fresh
```
