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

### Выход из системы
- **`POST /auth/logout`**
- **Описание:** Выполняет выход из системы, очищая аутентификационные cookie (`access_token`, `refresh_token`).
- **Тело запроса:** (пустое)
- **Ответ (`200 OK`):** Ответ содержит заголовки `Set-Cookie` для удаления cookie на стороне клиента.
- **Возможные ошибки:**
  - **`401 Unauthorized`**: Требуется аутентификация.

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

### Получение профиля пользователя
- **`GET /profile/{id}`**
- **Описание:** Возвращает детальную информацию о профиле пользователя, включая его навыки.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `ProfileDto`):**
  ```json
  {
    "id": "user-uuid-1",
    "study_group": "ИКБО-01-22",
    "telephone": "+79991234567",
    "roles": ["Initiator"],
    "email": "user@example.com",
    "last_name": "Иванов",
    "first_name": "Иван",
    "created_at": "2023-12-15T10:00:00",
    "skills": [
      {
        "id": "skill-uuid-1",
        "name": "Rust",
        "type": "Backend",
        "confirmed": true
      }
    ]
  }
  ```
- **Возможные ошибки:** `401 Unauthorized`, `404 Not Found`.

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

### Обновление навыков профиля
- **`PUT /profile/skills`**
- **Описание:** Обновляет список навыков текущего пользователя.
- **Права доступа:** Требуется аутентификация.
- **Тело запроса (`Vec<Uuid>`):**
  ```json
  [
    "skill-uuid-1",
    "skill-uuid-2"
  ]
  ```
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Успешное обновление навыков" }`
- **Возможные ошибки:** `401 Unauthorized`.

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

### Получение текущего пользователя
- **`GET /users`**
- **Описание:** Возвращает базовую информацию о текущем аутентифицированном пользователе.
- **Права доступа:** Требуется аутентификация.
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
- **Возможные ошибки:** `401 Unauthorized`.

### Получение пользователя по ID
- **`GET /users/{id}`**
- **Описание:** Возвращает базовую информацию о пользователе по его ID.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `UserDto`):** (Аналогично `GET /users`)
- **Возможные ошибки:** `401 Unauthorized`, `404 Not Found`.

### Получение списка всех пользователей
- **`GET /users/all`**
- **Описание:** Возвращает список пользователей с пагинацией.
- **Query параметры:** `?page=1&page_size=10`
- **Ответ (`200 OK`, `Vec<UserDto>`):** (Аналогично `GET /users`)
- **Возможные ошибки:** `401 Unauthorized`.

### Получение пользователей с их навыками
- **`GET /users/all/with-skills`**
- **Описание:** Возвращает список всех пользователей, включая их навыки.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<UserDto>`):**
  ```json
  [
    {
      "id": "user-uuid-1",
      "email": "user1@example.com",
      "last_name": "Иванов",
      "first_name": "Иван",
      // ...
      "skills": [
        {
          "id": "skill-uuid-1",
          "name": "Rust",
          "type": "Backend",
          "confirmed": true
        }
      ]
    }
  ]
  ```
- **Возможные ошибки:** `401 Unauthorized`.

### Получение пользователей, состоящих в командах
- **`GET /users/all/in-team`**
- **Описание:** Возвращает список всех пользователей, которые являются членами какой-либо команды.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<UserDto>`):** (Аналогично `GET /users`)
- **Возможные ошибки:** `401 Unauthorized`.

### Создание пользователя
- **`POST /users`**
- **Описание:** Создает нового пользователя без приглашения.
- **Права доступа:** `Admin`.
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
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Успешное создание пользователя" }`
- **Возможные ошибки:** `403 Forbidden`, `400 Bad Request`, `422 Unprocessable Entity`.

### Обновление пользователя
- **`PUT /users`**
- **Описание:** Обновляет данные любого пользователя по ID.
- **Права доступа:** `Admin`.
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
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Успешное обновление пользователя" }`
- **Возможные ошибки:** `403 Forbidden`, `404 Not Found`, `422 Unprocessable Entity`.

### Восстановление пользователя
- **`PUT /users/restore/{email}`**
- **Описание:** Восстанавливает "удаленного" (soft-deleted) пользователя.
- **Права доступа:** `Admin`.
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Успешное восстановление пользователя" }`
- **Возможные ошибки:** `403 Forbidden`, `404 Not Found`.

### Удаление пользователя
- **`DELETE /users/{id}`**
- **Описание:** Удаляет пользователя (soft-delete).
- **Права доступа:** `Admin`.
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Успешное удаление пользователя" }`
- **Возможные ошибки:** `403 Forbidden`, `404 Not Found`.

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
- **Права доступа:** `Admin`.
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
- **Возможные ошибки:** `403 Forbidden`.

---

## Company API (`/company`)

### Получение списка всех компаний
- **`GET /company`**
- **Описание:** Возвращает список всех компаний.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<CompanyResponse>`):**
  ```json
  [
    {
      "id": "company-uuid-1",
      "name": "My Company",
      "owner": {
        "id": "user-uuid-1",
        "email": "owner@example.com",
        "last_name": "Иванов",
        "first_name": "Иван",
        "study_group": "ИКБО-01-22",
        "telephone": "+79991234567",
        "roles": ["TeamOwner"],
        "created_at": "2023-12-15T10:00:00"
      },
      "members": []
    }
  ]
  ```
- **Возможные ошибки:** `401 Unauthorized`.

### Создание компании
- **`POST /company`**
- **Описание:** Создает новую компанию.
- **Права доступа:** `Admin`.
- **Тело запроса (`CreateCompanyRequest`):**
  ```json
  {
    "name": "New Awesome Company",
    "owner_id": "user-uuid-for-owner",
    "members": [
      "user-uuid-1",
      "user-uuid-2"
    ]
  }
  ```
- **Ответ (`200 OK`, `CompanyResponse`):** Возвращает созданную компанию.
- **Возможные ошибки:** `403 Forbidden`.

### Обновление компании
- **`PUT /company`**
- **Описание:** Обновляет данные компании по ID.
- **Права доступа:** `Admin`.
- **Тело запроса (`UpdateCompanyRequest`):**
  ```json
  {
    "id": "company-uuid-to-update",
    "name": "Updated Company Name",
    "owner_id": "new-owner-uuid",
    "members": ["user-uuid-3"]
  }
  ```
- **Ответ (`200 OK`, `CompanyResponse`):** Возвращает обновленную компанию.
- **Возможные ошибки:** `403 Forbidden`, `404 Not Found`.

### Получение компании по ID
- **`GET /company/{id}`**
- **Описание:** Возвращает детальную информацию о компании.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `CompanyResponse`):**
  ```json
  {
    "id": "company-uuid-1",
    "name": "My Company",
    "owner": { ... },
    "members": [ { ... } ]
  }
  ```
- **Возможные ошибки:** `401 Unauthorized`, `404 Not Found`.

### Удаление компании
- **`DELETE /company/{id}`**
- **Описание:** Удаляет компанию.
- **Права доступа:** `Admin`.
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Компания успешно удалена" }`
- **Возможные ошибки:** `403 Forbidden`, `404 Not Found`.

### Получение участников компании
- **`GET /company/{id}/members`**
- **Описание:** Возвращает список участников компании.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<UserDto>`):**
  ```json
  [
    {
      "id": "user-uuid-1",
      "email": "user1@example.com",
      ...
    }
  ]
  ```
- **Возможные ошибки:** `401 Unauthorized`.

### Получение моих компаний
- **`GET /company/my`**
- **Описание:** Возвращает список компаний, где текущий пользователь является владельцем или участником.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<CompanyResponse>`):** (Аналогично `GET /company`)
- **Возможные ошибки:** `401 Unauthorized`.

---

## Skill API (`/skill`)

### Получение всех навыков
- **`GET /skill`**
- **Описание:** Возвращает список всех навыков.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<SkillDto>`):**
  ```json
  [
    {
      "id": "skill-uuid-1",
      "name": "Rust",
      "type": "Backend",
      "confirmed": true,
      "creator_id": "user-uuid-1",
      "updater_id": null,
      "deleter_id": null
    }
  ]
  ```
- **Возможные ошибки:** `401 Unauthorized`.

### Получение своих и подтвержденных навыков
- **`GET /skill/my`**
- **Описание:** Возвращает `HashMap`, где ключ - это тип навыка, а значение - список подтвержденных навыков и навыков, созданных текущим пользователем.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `HashMap<String, Vec<SkillDto>>`):**
  ```json
  {
    "Backend": [
      {
        "id": "skill-uuid-1",
        "name": "Rust",
        "type": "Backend",
        "confirmed": true,
        "creator_id": "user-uuid-1"
      }
    ],
    "Frontend": [
      {
        "id": "skill-uuid-2",
        "name": "React",
        "type": "Frontend",
        "confirmed": false,
        "creator_id": "current-user-uuid"
      }
    ]
  }
  ```
- **Возможные ошибки:** `401 Unauthorized`.

### Получение навыков по типу
- **`GET /skill/type/{skill_type}`**
- **Описание:** Возвращает список навыков определенного типа (`Frontend`, `Backend`, `DevOps`, `Design`).
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<SkillDto>`):** (Аналогично `GET /skill`)
- **Возможные ошибки:** `401 Unauthorized`.

### Создание навыка
- **`POST /skill`**
- **Описание:** Создает новый навык. Если запрос от администратора, навык сразу помечается как `confirmed`.
- **Права доступа:** Требуется аутентификация.
- **Тело запроса (`CreateSkillRequest`):**
  ```json
  {
    "name": "New Skill",
    "type": "Backend"
  }
  ```
- **Ответ (`200 OK`, `SkillDto`):** Возвращает созданный навык.
- **Возможные ошибки:** `401 Unauthorized`, `400 Bad Request`.

### Обновление навыка
- **`PUT /skill`**
- **Описание:** Обновляет данные навыка.
- **Права доступа:** `Admin`.
- **Тело запроса (`UpdateSkillRequest`):**
  ```json
  {
    "id": "skill-uuid-to-update",
    "name": "Updated Skill Name",
    "type": "Frontend",
    "confirmed": true
  }
  ```
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Навык успешно обновлен" }`
- **Возможные ошибки:** `403 Forbidden`, `404 Not Found`.

### Удаление навыка
- **`DELETE /skill/{id}`**
- **Описание:** Удаляет навык.
- **Права доступа:** `Admin`.
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Навык успешно удален" }`
- **Возможные ошибки:** `403 Forbidden`, `404 Not Found`.

---

## Group API (`/group`)

### Получение списка всех групп
- **`GET /group`**
- **Описание:** Возвращает список всех групп с их участниками.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<GroupDto>`):**
  ```json
  [
    {
      "id": "group-uuid-1",
      "name": "Admins",
      "roles": ["Admin"],
      "members": [
        {
          "id": "user-uuid-1",
          "email": "admin@example.com",
          // ...
        }
      ]
    }
  ]
  ```
- **Возможные ошибки:** `401 Unauthorized`.

### Получение группы по ID
- **`GET /group/{id}`**
- **Описание:** Возвращает детальную информацию о группе.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `GroupDto`):**
  ```json
  {
    "id": "group-uuid-1",
    "name": "Admins",
    "roles": ["Admin"],
    "members": [ ... ]
  }
  ```
- **Возможные ошибки:** `401 Unauthorized`, `404 Not Found`.

### Создание группы
- **`POST /group`**
- **Описание:** Создает новую группу.
- **Права доступа:** `Admin`.
- **Тело запроса (`CreateGroupRequest`):**
  ```json
  {
    "name": "New Cool Group",
    "roles": ["TeamOwner"],
    "members": [
      "user-uuid-1",
      "user-uuid-2"
    ]
  }
  ```
- **Ответ (`200 OK`, `GroupDto`):** Возвращает созданную группу.
- **Возможные ошибки:** `403 Forbidden`.

### Обновление группы
- **`PUT /group`**
- **Описание:** Обновляет данные группы по ID.
- **Права доступа:** `Admin`.
- **Тело запроса (`UpdateGroupRequest`):**
  ```json
  {
    "id": "group-uuid-to-update",
    "name": "Updated Group Name",
    "roles": ["Initiator"],
    "members": ["user-uuid-3"]
  }
  ```
- **Ответ (`200 OK`, `GroupDto`):** Возвращает обновленную группу.
- **Возможные ошибки:** `403 Forbidden`, `404 Not Found`.

### Удаление группы
- **`DELETE /group/{id}`**
- **Описание:** Удаляет группу.
- **Права доступа:** `Admin`.
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Группа успешно удалена" }`
- **Возможные ошибки:** `403 Forbidden`, `404 Not Found`.

---

## Idea API (`/idea`)

### Сохранение идеи
- **`POST /idea`**
- **Описание:** Сохраняет новую или обновляет существующую идею. Для создания `id` должен быть `null`.
- **Права доступа:** `Initiator`, `Admin`.
- **Тело запроса (`SaveIdeaRequest`):**
  ```json
  {
    "id": "uuid-of-idea-optional",
    "name": "Название идеи",
    "status": "New",
    "problem": "Описание проблемы",
    // ...
    "max_team_size": 5,
    "min_team_size": 3
  }
  ```
- **Ответ (`200 OK`, `IdeaDto`):** Возвращает полную DTO сохраненной идеи.
- **Возможные ошибки:** `401 Unauthorized`, `403 Forbidden`, `422 Unprocessable Entity`.

### Получение всех идей
- **`GET /idea`**
- **Описание:** Возвращает список всех идей с пагинацией.
- **Query параметры:** `?page=1&page_size=10`
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<IdeaWithChecked>`):** Возвращает список идей. Поле `is_checked` показывает, просмотрена ли идея текущим пользователем (экспертом/ПО).
- **Возможные ошибки:** `401 Unauthorized`.

### Получение идеи по ID
- **`GET /idea/{id}`**
- **Описание:** Возвращает детальную информацию об идее по ID.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `IdeaWithChecked`):** Аналогично элементу в `GET /idea`.
- **Возможные ошибки:** `401 Unauthorized`, `404 Not Found`.

### Удаление идеи
- **`DELETE /idea/{id}`**
- **Описание:** Удаляет идею.
- **Права доступа:** `Initiator` (может удалить только свою идею), `Admin`.
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Идея успешно удалена" }`
- **Возможные ошибки:** `401 Unauthorized`, `403 Forbidden`, `404 Not Found`.

### Получение идей инициатора
- **`GET /idea/initiator`**
- **Описание:** Возвращает список всех идей, созданных текущим пользователем.
- **Query параметры:** `?page=1&page_size=10`
- **Права доступа:** `Initiator`.
- **Ответ (`200 OK`, `Vec<IdeaWithChecked>`):** (Аналогично `GET /idea`)
- **Возможные ошибки:** `401 Unauthorized`.

### Получение идей на согласовании
- **`GET /idea/on-confirmation`**
- **Описание:** Возвращает список идей на согласовании для экспертов и ПО.
- **Query параметры:** `?page=1&page_size=10`
- **Права доступа:** `ProjectOffice`, `Expert`, `Admin`.
- **Ответ (`200 OK`, `Vec<IdeaWithChecked>`):** (Аналогично `GET /idea`)
- **Возможные ошибки:** `401 Unauthorized`, `403 Forbidden`.

### Обновление статуса идеи
- **`PUT /idea/status`**
- **Описание:** Обновляет статус идеи.
- **Права доступа:** `ProjectOffice`, `Expert`, `Admin`.
- **Тело запроса (`IdeaStatusRequest`):** `{ "id": "...", "status": "Approved" }`
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Статус идеи успешно обновлен" }`
- **Возможные ошибки:** `401 Unauthorized`, `403 Forbidden`.

### Отправка идеи на согласование
- **`PUT /idea/send/{id}`**
- **Описание:** Инициатор отправляет свою идею на согласование (статус `OnApproval`).
- **Права доступа:** `Initiator`.
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Идея успешно отправлена на согласование" }`
- **Возможные ошибки:** `401 Unauthorized`, `403 Forbidden`, `404 Not Found`.

### Получение и сохранение навыков идеи
- **`GET /idea/skills/{id}`**: Возвращает список навыков для идеи.
- **`POST /idea/skills`**: Обновляет список навыков для идеи.
  - **Права доступа:** `Initiator`, `Admin`.
  - **Тело запроса (`IdeaSkillRequest`):** `{ "id": "...", "skills": [{...}] }`
  - **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Навыки для идеи успешно обновлены" }`

---

## Team API (`/team`)

### Получение списка команд
- **`GET /team`**
- **Описание:** Возвращает список всех команд.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<TeamDto>`):**
  ```json
  [
    {
      "id": "team-uuid-1",
      "name": "Команда 1",
      "description": "Описание команды",
      "is_closed": false,
      "has_active_project": false,
      "created_at": "2023-12-15T10:00:00",
      "owner": { "id": "...", "email": "...", "last_name": "...", "first_name": "..." },
      "leader": { "id": "...", "email": "...", "last_name": "...", "first_name": "..." },
      "members": [ { ... } ],
      "wanted_skills": [ { ... } ],
      "member_skills": [ { ... } ],
      "members_count": 3,
      "is_refused": false
    }
  ]
  ```
- **Возможные ошибки:** `401 Unauthorized`.

### Получение списка своих команд
- **`GET /team/my/{idea_id}`**
- **Описание:** Возвращает список команд, в которых состоит текущий пользователь, и которые могут быть привязаны к идее.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<TeamDto>`):** (Аналогично `GET /team`)
- **Возможные ошибки:** `401 Unauthorized`.

### Получение команды по ID
- **`GET /team/{id}`**
- **Описание:** Возвращает детальную информацию о команде.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `TeamDto`):** (Аналогично элементу в `GET /team`)
- **Возможные ошибки:** `401 Unauthorized`, `404 Not Found`.

### Создание команды
- **`POST /team`**
- **Описание:** Создает новую команду.
- **Права доступа:** `Admin`, `TeamOwner`.
- **Тело запроса (`CreateTeamRequest`):**
  ```json
  {
    "name": "Новая команда",
    "description": "Описание новой команды",
    "is_closed": false,
    "owner_id": "owner-user-uuid",
    "leader_id": "leader-user-uuid", // опционально
    "members": ["member-user-uuid-1"],
    "wanted_skills": ["skill-uuid-1"]
  }
  ```
- **Ответ (`200 OK`, `TeamDto`):** Возвращает созданную команду.
- **Возможные ошибки:** `401 Unauthorized`, `403 Forbidden`.

### Обновление команды
- **`PUT /team`**
- **Описание:** Обновляет данные команды.
- **Права доступа:** `Admin`, `TeamOwner`, `TeamLeader`.
- **Тело запроса (`UpdateTeamRequest`):**
  ```json
  {
    "id": "team-uuid-to-update",
    "name": "Обновленное название",
    "description": "Обновленное описание",
    "is_closed": true,
    "wanted_skills": ["skill-uuid-2"]
  }
  ```
- **Ответ (`200 OK`, `TeamDto`):** Возвращает обновленную команду.
- **Возможные ошибки:** `401 Unauthorized`, `403 Forbidden`, `404 Not Found`.

---

## Rating API (`/rating`)

### Получение всех рейтингов для идеи
- **`GET /rating/{idea_id}`**
- **Описание:** Возвращает список всех оценок для указанной идеи.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<RatingDto>`):**
  ```json
  [
    {
      "id": "rating-uuid-1",
      "expert": { ... },
      "idea_id": "idea-uuid-1",
      "market_value": 5,
      // ...
      "rating": 3.8,
      "is_confirmed": false
    }
  ]
  ```
- **Возможные ошибки:** `401 Unauthorized`.

### Получение рейтингов эксперта для идеи
- **`GET /rating/expert/{idea_id}`**
- **Описание:** Возвращает оценки текущего эксперта для идеи.
- **Права доступа:** `Expert`.
- **Ответ (`200 OK`, `Vec<RatingDto>`):** (Аналогично `GET /rating/{idea_id}`)
- **Возможные ошибки:** `401 Unauthorized`, `403 Forbidden`.

### Сохранение/Подтверждение рейтинга
- **`PUT /rating/save`**: Сохраняет или обновляет оценку.
- **`PUT /rating/confirm`**: Сохраняет и подтверждает оценку.
- **Права доступа:** `Admin`, `Expert`, `ProjectOffice`.
- **Тело запроса (`UpdateRatingRequest`):**
  ```json
  {
    "id": "rating-uuid-to-update",
    "market_value": 5,
    "originality": 4,
    "technical_realizability": 3,
    "suitability": 5,
    "budget": 2
  }
  ```
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Рейтинг успешно сохранен/подтвержден" }`
- **Возможные ошибки:** `401 Unauthorized`, `403 Forbidden`.

---

## Market API (`/market`)

### Получение списка всех маркетов
- **`GET /market`**
- **Описание:** Возвращает список всех маркетов.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<MarketDto>`):**
  ```json
  [
    {
      "id": "market-uuid-1",
      "name": "Market 1",
      "start_date": "2026-01-01",
      "finish_date": "2026-02-01",
      "status": "NEW"
    }
  ]
  ```
- **Возможные ошибки:** `401 Unauthorized`.

### Получение списка активных маркетов
- **`GET /market/active`**
- **Описание:** Возвращает список маркетов со статусом `ACTIVE`.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `Vec<MarketDto>`):** (Аналогично `GET /market`)
- **Возможные ошибки:** `401 Unauthorized`.

### Получение маркета по ID
- **`GET /market/{id}`**
- **Описание:** Возвращает детальную информацию о маркете.
- **Права доступа:** Требуется аутентификация.
- **Ответ (`200 OK`, `MarketDto`):** (Аналогично элементу в `GET /market`)
- **Возможные ошибки:** `401 Unauthorized`, `404 Not Found`.

### Создание/Обновление маркета
- **`POST /market`**: Создает новый маркет.
  - **Тело запроса (`CreateMarketRequest`):** `{ "name": "...", "start_date": "...", "finish_date": "..." }`
- **`PUT /market`**: Обновляет данные маркета.
  - **Тело запроса (`UpdateMarketRequest`):** `{ "id": "...", "name": "...", ... }`
- **Права доступа:** `Admin`, `ProjectOffice`.
- **Ответ (`200 OK`, `MarketDto`):** Возвращает созданный/обновленный маркет.
- **Возможные ошибки:** `403 Forbidden`, `404 Not Found`.

### Обновление статуса маркета
- **`PUT /market/status`**
- **Описание:** Обновляет статус маркета.
- **Права доступа:** `Admin`, `ProjectOffice`.
- **Тело запроса (`UpdateMarketStatusRequest`):** `{ "id": "...", "status": "ACTIVE" }`
- **Ответ (`200 OK`, `MarketDto`):** Возвращает обновленный маркет.
- **Возможные ошибки:** `403 Forbidden`, `404 Not Found`.

### Удаление маркета
- **`DELETE /market/{id}`**
- **Описание:** Удаляет маркет.
- **Права доступа:** `Admin`, `ProjectOffice`.
- **Ответ (`200 OK`, `MessageResponse`):** `{ "message": "Маркет успешно удален" }`
- **Возможные ошибки:** `403 Forbidden`, `404 Not Found`.
