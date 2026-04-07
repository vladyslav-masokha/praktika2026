# PulseCommerce

![Rust](https://img.shields.io/badge/backend-Rust-orange?style=for-the-badge&logo=rust)
![React](https://img.shields.io/badge/frontend-React-blue?style=for-the-badge&logo=react)
![TypeScript](https://img.shields.io/badge/language-TypeScript-3178C6?style=for-the-badge&logo=typescript)
![PostgreSQL](https://img.shields.io/badge/database-PostgreSQL-336791?style=for-the-badge&logo=postgresql)
![RabbitMQ](https://img.shields.io/badge/broker-RabbitMQ-FF6600?style=for-the-badge&logo=rabbitmq)
![Docker](https://img.shields.io/badge/deploy-Docker-2496ED?style=for-the-badge&logo=docker)

## PulseCommerce — розподілена event-driven платформа для оформлення та обробки онлайн-замовлень

**PulseCommerce** — це навчальний full-stack проєкт, реалізований у межах виробничої практики.  
Система побудована на основі **мікросервісної архітектури**, використовує **асинхронну взаємодію через RabbitMQ**, окремі сервіси для замовлень, оплати та сповіщень, а також **React + TypeScript** фронтенд для роботи користувачів та адміністратора.

Проєкт демонструє сучасний підхід до побудови масштабованих систем електронної комерції з поділом відповідальності між сервісами, підтримкою WebSocket-сповіщень та аналізом передачі подій у форматах **JSON** і **Protocol Buffers**.

---

## Зміст

- [Основні можливості](#основні-можливості)
- [Архітектура системи](#архітектура-системи)
- [Діаграми](#діаграми)
- [Стек технологій](#стек-технологій)
- [Структура проєкту](#структура-проєкту)
- [Як запустити проєкт](#як-запустити-проєкт)
- [Налаштування змінних середовища](#налаштування-змінних-середовища)
- [Основні API-маршрути](#основні-api-маршрути)
- [Як працює система](#як-працює-система)
- [Порівняння JSON та Protobuf](#порівняння-json-та-protobuf)
- [Переваги проєкту](#переваги-проєкту)
- [Подальший розвиток](#подальший-розвиток)
- [Автори](#автори)

---

## Основні можливості

### Для користувача

- перегляд каталогу товарів;
- оформлення нового замовлення;
- відстеження статусу замовлення;
- перегляд історії замовлень;
- отримання сповіщень у реальному часі через WebSocket.

### Для адміністратора

- перегляд статистики системи;
- керування товарами;
- керування користувачами;
- перегляд замовлень;
- моніторинг активності платформи.

### На рівні системи

- API Gateway як єдина точка входу;
- окремий Order Service для створення та обробки замовлень;
- окремий Payment Service для емуляції та обробки оплати;
- Notification Service для WebSocket-сповіщень;
- RabbitMQ для event-driven взаємодії між сервісами;
- PostgreSQL для збереження даних;
- Benchmark Service для порівняння JSON і Protobuf.

---

## Архітектура системи

PulseCommerce побудований за принципом **мікросервісної архітектури**.

### До складу системи входять:

- **Frontend (React + TypeScript)** — інтерфейс користувача та адміністратора;
- **API Gateway** — приймає HTTP-запити з клієнта та маршрутизує їх до внутрішніх сервісів;
- **Order Service** — створює замовлення, зберігає їх у БД та публікує події;
- **Payment Service** — обробляє події оплати та змінює статус транзакцій;
- **Notification Service** — передає оновлення клієнту через WebSocket;
- **RabbitMQ** — брокер повідомлень для обміну подіями між сервісами;
- **PostgreSQL** — база даних для сервісів;
- **Benchmark Service** — сервіс для тестування форматів передачі повідомлень.

---

## Діаграми

### C4 Model — Level 1 (System Context)

![C4 Level 1](./diagrams/c4-level-1-system-context.png)

На цьому рівні показано взаємодію користувача з системою PulseCommerce як єдиною платформою онлайн-замовлень.

### C4 Model — Level 2 (Container Diagram)

![C4 Level 2](./diagrams/c4-level-2-container-diagram.png)

На діаграмі контейнерів показано розподіл системи на фронтенд, API Gateway, мікросервіси, брокер повідомлень і бази даних.

### C4 Model — Level 3 (Order Service Components)

![C4 Level 3](./diagrams/c4-level-3-order-service-component.png)

На цьому рівні деталізовано внутрішню структуру `Order Service`: контролери, бізнес-логіка, репозиторії, робота з БД та публікація подій.

### UML Sequence Diagram — обробка замовлення

![Sequence Diagram](./diagrams/sequence-order-processing.png)

Діаграма послідовності демонструє повний сценарій створення замовлення, відправлення події, обробки оплати та надсилання сповіщення користувачу.

### Benchmark / Event Flow Diagram

![Benchmark Diagram](./diagrams/benchmark-json-vs-protobuf.png)

Ця діаграма ілюструє порівняння JSON та Protobuf у межах benchmark-сервісу.

---

## Стек технологій

### Frontend

- React
- TypeScript
- Vite
- Tailwind CSS

### Backend

- Rust
- Axum
- Tokio
- SQLx
- Reqwest
- Tower HTTP

### Інфраструктура

- PostgreSQL
- RabbitMQ
- Docker
- Docker Compose
- WebSocket

### Формати передачі даних

- JSON
- Protocol Buffers

---

## Структура проєкту

```bash
praktika2026/
├── api-gateway/
├── order-service/
├── payment-service/
├── notification-service/
├── benchmark-service/
├── shared/
├── proto/
├── Cargo.toml
└── docker-compose.yml
│
├── frontend/
│   ├── src/
│   ├── public/
│   ├── package.json
│   └── vite.config.ts
│
├── diagrams/
│   ├── c4-level-1-system-context.png
│   ├── c4-level-2-container-diagram.png
│   ├── c4-level-3-order-service-component.png
│   ├── sequence-order-processing.png
│   └── benchmark-json-vs-protobuf.png
│
└── README.md
```

---

## Як запустити проєкт

### 1. Клонувати репозиторій

```bash
git clone https://github.com/vladyslav-masokha/praktika2026.git
cd praktika2026
```

### 2. Запустити інфраструктуру та backend через Docker Compose

Перейдіть у папку `backend` і виконайте:

```bash
cd backend
docker compose up --build
```

Після запуску будуть доступні:

- API Gateway — `http://localhost:8080`
- Order Service — `http://localhost:8081`
- Payment Service — `http://localhost:8082`
- Notification Service — `http://localhost:8083`
- RabbitMQ Management — `http://localhost:15672`

### 3. Запустити frontend

Перейдіть у папку `frontend`:

```bash
cd frontend
npm install
npm run dev
```

Frontend буде доступний за адресою:

```bash
http://localhost:5173
```

---

## Налаштування змінних середовища

Для коректної роботи потрібно створити файли `.env` у `backend` та `frontend`.

### Приклад для backend

```env
API_GATEWAY_PORT=8080
ORDER_SERVICE_PORT=8081
PAYMENT_SERVICE_PORT=8082
NOTIFICATION_SERVICE_PORT=8083

ORDER_SERVICE_URL=http://order-service:8081
PAYMENT_SERVICE_URL=http://payment-service:8082
NOTIFICATION_SERVICE_URL=http://notification-service:8083

ORDER_DATABASE_URL=postgres://postgres:postgres@postgres:5432/order_db
PAYMENT_DATABASE_URL=postgres://postgres:postgres@postgres:5432/payment_db

RABBITMQ_URL=amqp://guest:guest@rabbitmq:5672/%2f
AMQP_EXCHANGE=ecommerce.events

JWT_SECRET=your_super_secret_key
CORS_ALLOWED_ORIGINS=http://localhost:5173,http://127.0.0.1:5173
```

### Приклад для frontend

```env
VITE_API_BASE_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8080/ws
VITE_ORDER_HEALTH_URL=http://localhost:8081/health
VITE_PAYMENT_HEALTH_URL=http://localhost:8082/health
VITE_NOTIFICATION_HEALTH_URL=http://localhost:8083/health
```

---

## Основні API-маршрути

### API Gateway

- `GET /health` — перевірка доступності gateway
- `POST /api/auth/register` — реєстрація користувача
- `POST /api/auth/login` — авторизація
- `GET /api/products` — отримання списку товарів
- `POST /api/orders` — створення замовлення
- `GET /api/orders` — історія замовлень користувача
- `GET /api/admin/stats` — статистика для адміністратора
- `GET /api/admin/products` — список товарів для адмінки
- `POST /api/admin/products` — створення товару
- `PUT /api/admin/products/{id}` — редагування товару
- `DELETE /api/admin/products/{id}` — видалення товару

### WebSocket

- `GET /ws` — канал для отримання оновлень у реальному часі

---

## Як працює система

1. Користувач відкриває frontend та переглядає каталог товарів.
2. Після оформлення покупки frontend надсилає запит до API Gateway.
3. API Gateway передає його до Order Service.
4. Order Service створює замовлення в базі даних.
5. Після створення замовлення Order Service публікує подію в RabbitMQ.
6. Payment Service отримує подію та виконує обробку оплати.
7. Після завершення оплати формується нова подія про зміну статусу.
8. Notification Service отримує подію та надсилає повідомлення клієнту через WebSocket.
9. Frontend у реальному часі оновлює статус замовлення.

---

## Порівняння JSON та Protobuf

У проєкті реалізовано окремий benchmark-сервіс для порівняння двох форматів передачі даних:

- **JSON** — простий для читання, зручний для налагодження;
- **Protocol Buffers** — компактніший, швидший і ефективніший для передачі подій.

### Приклад результатів benchmark

**JSON:**

- середня затримка: ~155 мс
- середній розмір повідомлення: ~133 байти

**Protobuf:**

- середня затримка: ~136 мс
- середній розмір повідомлення: ~67 байт

Отримані результати показують, що **Protobuf** є більш ефективним для event-driven систем із великим навантаженням.

---

## Переваги проєкту

- сучасна мікросервісна архітектура;
- розділення відповідальності між сервісами;
- масштабованість;
- асинхронна взаємодія через брокер повідомлень;
- підтримка WebSocket-сповіщень;
- порівняння форматів передачі даних;
- зручний інтерфейс користувача та адміністратора;
- можливість подальшого розширення функціоналу.

---

## Подальший розвиток

У майбутньому проєкт можна розширити:

- інтеграцією реальної платіжної системи;
- додаванням ролей і прав доступу;
- впровадженням observability-інструментів;
- логуванням та трасуванням подій;
- використанням Kubernetes для оркестрації;
- CI/CD для автоматичного деплою;
- аналітикою продажів та поведінки користувачів.

---

## Автори

- **Владислав Масоха**
- **Дар'я Якименко**

Навчальний проєкт у межах виробничої практики.  
GitHub: [vladyslav-masokha](https://github.com/vladyslav-masokha)
