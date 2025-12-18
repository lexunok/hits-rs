# Документация по API

Этот документ описывает основные эндпоинты API, их назначение, необходимые права доступа и структуры данных для взаимодействия с фронтендом.

## Базовые URL
Все пути API относительны и начинаются на `/api`.

## Аутентификация
- Аутентификация происходит через JWT-токены.
- `accessToken` передается в заголовке `Authorization` как `Bearer <token>`.
- `refreshToken` передается в cookie с именем `refresh_token`.
- При получении статуса `401 Unauthorized` с ошибкой `Invalid token`, фронтенд должен выполнить запрос на эндпоинт `POST /auth/refresh` для обновления токенов.

## Общие ошибки
- **`500 Internal Server Error`**: Общая ошибка сервера. Может возникнуть из-за проблем с базой данных (`DbErr`), Redis (`RedisErr`) или другой внутренней логикой.
- **`401 Unauthorized` (`Invalid token`)**: Предоставленный `accessToken` недействителен или истек. Требуется обновление токенов.
- **`403 Forbidden`**: У пользователя нет необходимых прав для выполнения операции. В основном касается эндпоинтов администратора.

---

##  Auth API (`/auth`)

### Вход в систему
- **`POST /auth/login`**
- **Описание:** Аутентифицирует пользователя и возвращает пару токенов.
- **Тело запроса (`LoginPayload`):**
  ```json
  {
    "email": "user@example.com",
    "password": "password123"
  }
  ```
- **Ответ (`200 OK`):** `accessToken` и `refreshToken` устанавливаются автоматически.
- **Возможные ошибки:**
  - **`401 Unauthorized` (`Wrong credentials`)**: Неверный email или пароль.
  - **`422 Unprocessable Entity`**: Ошибка валидации. Некорректный формат email или пароль короче 8 символов.
  - **`500 Internal Server Error` (`Token creation error`)**: Ошибка при создании токена.

### Регистрация по приглашению
- **`POST /auth/registration/{invitation_id}`**
- **Описание:** Регистрирует нового пользователя на основе существующего приглашения.
- **Тело запроса (`RegisterPayload`):**
  ```json
  {
    "email": "newuser@example.com",
    "password": "password123",
    "last_name": "Иванов",
    "first_name": "Иван",
    "study_group": "ИКБО-01-22", // опционально
    "telephone": "+79991234567"  // опционально
  }
  ```
- **Ответ (`200 OK`):** Возвращает `accessToken` и `refreshToken`.
- **Возможные ошибки:**
  - **`404 Not Found`**: Приглашение с указанным `invitation_id` не найдено.
  - **`400 Bad Request` (`Custom`)**: Email в запросе не совпадает с email в приглашении, или пользователь с таким email уже существует.
  - **`422 Unprocessable Entity`**: Ошибки валидации полей (email, пароль и т.д.).
  - **`500 Internal Server Error` (`Token creation error`)**: Ошибка при создании токена.

### Обновление токенов
- **`POST /auth/refresh`**
- **Описание:** Обновляет `accessToken` и `refreshToken`.
- **Тело запроса:** (пустое)
- **Ответ (`200 OK`):** Устанавливает новую пару токенов.
- **Возможные ошибки:**
  - **`401 Unauthorized` (`Invalid token`)**: `refreshToken` в cookie отсутствует или недействителен.

### Запрос на сброс пароля
- **`POST /auth/password/verification/{email}`**
- **Описание:** Инициирует процедуру сброса пароля.
- **Ответ (`200 OK`, `IdResponse`):**
  ```json
  {
    "id": "uuid-of-verification-code"
  }
  ```
- **Возможные ошибки:**
  - **`404 Not Found`**: Пользователь с таким email не найден.

### Подтверждение сброса пароля
- **`PUT /auth/password`**
- **Описание:** Устанавливает новый пароль.
- **Тело запроса (`PasswordResetPayload`):**
  ```json
  {
    "id": "uuid-of-verification-code",
    "code": "123456",
    "password": "newStrongPassword123"
  }
  ```
- **Ответ (`200 OK`, `MessageResponse`):**
  ```json
  {
    "message": "Успешное обновление пароля"
  }
  ```
- **Возможные ошибки:**
  - **`400 Bad Request` (`Custom`)**: Неверный код верификации.
  - **`422 Unprocessable Entity`**: Ошибки валидации (код не 6 цифр, пароль не менее 8 символов).

---

## Profile API (`/profile`)

### Обновление профиля
- **`PUT /profile`**
- **Описание:** Обновляет данные текущего пользователя.
- **Тело запроса (`ProfileUpdatePayload`):**
  ```json
  {
    "last_name": "Петров",
    "first_name": "Петр",
    "study_group": "ИКБО-02-22", // опционально
    "telephone": "+79997654321"  // опционально
  }
  ```
- **Ответ (`200 OK`, `MessageResponse`):**
  ```json
  {
    "message": "Успешное обновление профиля"
  }
  ```
- **Возможные ошибки:**
  - **`401 Unauthorized`**: Требуется аутентификация.

### Запрос на смену email
- **`POST /profile/email/verification/{new_email}`**
- **Описание:** Инициирует смену email.
- **Ответ (`200 OK`, `IdResponse`):**
  ```json
  {
    "id": "uuid-of-verification-code"
  }
  ```
- **Возможные ошибки:**
  - **`401 Unauthorized`**: Требуется аутентификация.
  - **`400 Bad Request` (`Custom`)**: Пользователь с таким `new_email` уже существует.

### Подтверждение смены email
- **`PUT /profile/email`**
- **Описание:** Подтверждает смену email.
- **Тело запроса (`EmailResetPayload`):**
  ```json
  {
    "id": "uuid-of-verification-code",
    "code": "123456"
  }
  ```
- **Ответ (`200 OK`, `MessageResponse`):**
  ```json
  {
    "message": "Успешное обновление почты"
  }
  ```
- **Возможные ошибки:**
  - **`401 Unauthorized`**: Требуется аутентификация.
  - **`400 Bad Request` (`Custom`)**: Неверный код верификации.
  - **`422 Unprocessable Entity`**: Код должен состоять из 6 цифр.

### Загрузка аватара
- **`POST /profile/avatar`**
- **Описание:** Загружает или обновляет аватар текущего пользователя. Ожидает `multipart/form-data` с полем `avatar`, содержащим файл изображения.
- **Тело запроса (`multipart/form-data`):**
  Поле `avatar` должно содержать файл изображения (например, PNG, JPEG).
- **Ответ (`200 OK`, `MessageResponse`):**
  ```json
  {
    "message": "Аватар успешно обновлен"
  }
  ```
- **Возможные ошибки:**
  - **`401 Unauthorized`**: Требуется аутентификация.
  - **`400 Bad Request`**: Файл аватара не предоставлен или недействителен (например, поле `avatar` отсутствует).

### Получение аватара
- **`GET /images/avatar/{user_id}.webp`**
- **Описание:** Возвращает аватар пользователя по его ID в формате WebP.
- **Ответ (`200 OK`, `image/webp`):** Бинарные данные изображения в формате WebP.
- **Возможные ошибки:**
  - **`404 Not Found`**: Аватар пользователя с указанным ID не найден.

---

## Users API (`/users`)

### Получение списка пользователей
- **`GET /users`**
- **Описание:** Возвращает список пользователей с пагинацией.
- **Query параметры:** `?page=1&page_size=10`
- **Ответ (`200 OK`, `Vec<UserDto>`):**
  ```json
  [
    {
      "id": "user-uuid-1",
      "email": "user1@example.com",
      "last_name": "Иванов",
      "first_name": "Иван",
      "study_group": "ИКБО-01-22",
      "telephone": "+79991234567",
      "roles": ["Initiator"],
      "created_at": "2023-12-15T10:00:00"
    }
  ]
  ```
- **Возможные ошибки:**
  - **`401 Unauthorized`**: Требуется аутентификация.

### Получение пользователя по ID
- **`GET /users/{id}`**
- **Описание:** Возвращает детальную информацию о пользователе.
- **Ответ (`200 OK`, `UserDto`):**
  ```json
  {
    "id": "user-uuid-1",
    "email": "user1@example.com",
    "last_name": "Иванов",
    "first_name": "Иван",
    "study_group": "ИКБО-01-22",
    "telephone": "+79991234567",
    "roles": ["Initiator"],
    "created_at": "2023-12-15T10:00:00"
  }
  ```
- **Возможные ошибки:**
  - **`401 Unauthorized`**: Требуется аутентификация.
  - **`404 Not Found`**: Пользователь с указанным `id` не найден.
  
### Создание пользователя
- **`POST /users`**
- **Описание:** Создает нового пользователя без приглашения.
- **Тело запроса (`UserCreatePayload`):**
  ```json
  {
    "email": "admincreated@example.com",
    "password": "password123",
    "last_name": "Сидоров",
    "first_name": "Сидор",
    "roles": ["Admin"],
    "study_group": null,
    "telephone": null
  }
  ```
- **Ответ (`200 OK`, `MessageResponse`):**
  ```json
  {
    "message": "Успешное создание пользователя"
  }
  ```
- **Возможные ошибки:**
  - **`403 Forbidden`**: Нет прав администратора.
  - **`400 Bad Request` (`Custom`)**: Пользователь с таким email уже существует.
  - **`422 Unprocessable Entity`**: Ошибки валидации полей.

### Обновление пользователя
- **`PUT /users`**
- **Описание:** Обновляет данные любого пользователя по ID.
- **Тело запроса (`UserUpdatePayload`):**
  ```json
  {
    "id": "user-uuid-to-update",
    "email": "updated@example.com",
    "last_name": "Петров",
    "first_name": "Петр",
    "roles": ["TeamOwner"],
    "study_group": "ИКБО-03-22",
    "telephone": "+79990000000"
  }
  ```
- **Ответ (`200 OK`, `MessageResponse`):**
  ```json
  {
    "message": "Успешное обновление пользователя"
  }
  ```
- **Возможные ошибки:**
  - **`403 Forbidden`**: Нет прав администратора.
  - **`404 Not Found`**: Пользователь не найден.
  - **`422 Unprocessable Entity`**: Ошибки валидации полей.

### Восстановление пользователя
- **`PUT /users/restore/{email}`**
- **Описание:** Восстанавливает "удаленного" (soft-deleted) пользователя.
- **Ответ (`200 OK`, `MessageResponse`):**
  ```json
  {
    "message": "Успешное восстановление пользователя"
  }
  ```
- **Возможные ошибки:**
  - **`403 Forbidden`**: Нет прав администратора.
  - **`404 Not Found`**: Пользователь не найден.

### Удаление пользователя
- **`DELETE /users/{id}`**
- **Описание:** Удаляет пользователя (soft-delete).
- **Ответ (`200 OK`, `MessageResponse`):**
  ```json
  {
    "message": "Успешное удаление пользователя"
  }
  ```
- **Возможные ошибки:**
  - **`403 Forbidden`**: Нет прав администратора.
  - **`404 Not Found`**: Пользователь не найден.

---

## Invitation API (`/invitation`)

### Получение информации о приглашении
- **`GET /invitation/{id}`**
- **Описание:** Возвращает email и ID приглашения.
- **Ответ (`200 OK`, `InvitationResponse`):**
  ```json
  {
    "email": "invited@example.com",
    "code": "uuid-of-invitation"
  }
  ```
- **Возможные ошибки:**
  - **`404 Not Found`**: Приглашение не найдено.
  
### Отправка приглашений
- **`POST /invitation`**
- **Описание:** Отправляет приглашения на указанные email с заданными ролями.
- **Тело запроса (`InvitationPayload`):**
  ```json
  {
    "emails": ["user1@example.com", "user2@example.com"],
    "roles": ["Initiator"]
  }
  ```
- **Ответ (`200 OK`, `MessageResponse`):**
  ```json
  {
    "message": "Новые приглашения успешно отправлены в кол-ве 2"
  }
  ```
- **Возможные ошибки:**
  - **`403 Forbidden`**: Нет прав администратора.
