# Backend API for User and Transaction Management

## Overview

This backend API provides functionality for user management, account balance retrieval, and transaction management. The service is built using Rust, with PostgreSQL as the database. It includes JWT-based authentication for secure access and supports operations like deposits, withdrawals, and transfers.

---

## Technologies Used

- **Rust**: Backend programming language.
- **PostgreSQL**: Relational database.
- **Docker**: Containerization for easy setup and deployment.
- **Docker Compose**: Infrastructure orchestration.

---

## Features

- **User Management**:
  - Users can register themselves, and an account is automatically created.
  - Fetch user details and update user profiles.
- **Account Management**:
  - Retrieve account balance details.
- **Transaction Management**:
  - Perform deposit, withdrawal, and transfer operations.
  - View transaction history and fetch details of individual transactions.
- **Authentication**:
  - JWT-based authentication ensures secure access to APIs.

---

## Database Design

### Tables

#### 1. **Users**

| Attribute   | Data Type | Description               |
| ----------- | --------- | ------------------------- |
| Id          | Number    | Primary key               |
| user\_id    | UUID      | Unique user identifier    |
| username    | String    | User's name               |
| email       | String    | User's email              |
| password    | String    | Hashed user password      |
| created\_at | DateTime  | Record creation timestamp |
| updated\_at | DateTime  | Record update timestamp   |

#### 2. **Account Balance**

| Attribute   | Data Type | Description               |
| ----------- | --------- | ------------------------- |
| Id          | Number    | Primary key               |
| account\_id | UUID      | Unique account identifier |
| user\_id    | UUID      | Associated user ID        |
| balance     | Decimal   | Account balance amount    |
| created\_at | DateTime  | Record creation timestamp |
| updated\_at | DateTime  | Record update timestamp   |

#### 3. **Transactions**

| Attribute         | Data Type | Description                   |
| ----------------- | --------- | ----------------------------- |
| Id                | Number    | Primary key                   |
| transaction\_id   | UUID      | Unique transaction identifier |
| sender\_id        | String    | Sender's user ID              |
| receiver\_id      | String    | Receiver's user ID            |
| amount            | Decimal   | Transaction amount            |
| transaction\_type | String    | Type of transaction           |
| status            | String    | Transaction status            |
| created\_at       | DateTime  | Record creation timestamp     |
| updated\_at       | DateTime  | Record update timestamp       |

---

## API Endpoints

### User Management

| Method | API                  | Authentication | Request Example                                                     | Response Example                                                                       |
| ------ | -------------------- | -------------- | ------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| POST   | /user/register\_user | N/A            | `{ "username":"test", "email":"test@test.com", "password":"test" }` | `{ "message": "User Registration Successfully", "status": "Success" }`                 |
| GET    | /user/get\_token     | N/A            | `{ "email":"test@test.com", "password":"test" }`                    | `{ "message": "Successfully logged in", "status": "Success", "token": "<JWT_TOKEN>" }` |
| GET    | /user/get\_user      | Bearer Token   | `{ "user_id":"be296e10-7c91-485d-a5fa-4cb8a949d4f7" }`              | `{ "user_id": "be296e10-7c91-485d-a5fa-4cb8a949d4f7", "username": "test_updated" }`    |
| POST   | /user/update\_user   | Bearer Token   | `{ "username":"test_updated" }`                                     | `{ "message": "User Updated Successfully", "status": "Success" }`                      |

### Account Management

| Method | API                     | Authentication | Request Example | Response Example                                                             |
| ------ | ----------------------- | -------------- | --------------- | ---------------------------------------------------------------------------- |
| GET    | /balance/fetch\_balance | Bearer Token   | N/A             | `{ "user_id": "be296e10-7c91-485d-a5fa-4cb8a949d4f7", "balance": "100.00" }` |

### Transaction Management

| Method | API                             | Authentication | Request Example                                                      | Response Example                                                                                  |
| ------ | ------------------------------- | -------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| POST   | /transaction/operation          | Bearer Token   | `{ "receiver":null, "amount":100.00, "transaction_type":"deposit" }` | `{ "message": "Transaction added successfully", "status": "Success" }`                            |
| GET    | /transaction/fetch\_transaction | Bearer Token   | `{ "transaction_id":"21fb8729-a50d-4d96-aec1-6f346e721d59" }`        | `{ "transaction_id": "21fb8729-a50d-4d96-aec1-6f346e721d59", "transaction_type": "deposit" }`     |
| GET    | /transaction/list\_trans        | Bearer Token   | N/A                                                                  | `[ { "transaction_id": "21fb8729-a50d-4d96-aec1-6f346e721d59", "transaction_type": "deposit" } ]` |

---

## Steps to Run the Project

### Prerequisites

- Docker and Docker Compose installed.
- PostgreSQL instance running.
- Environment variable `DATABASE_URL` configured with the PostgreSQL connection string.

### Run Using Docker Compose

1. Clone the repository.

   ```bash
   git clone <repository-url>
   cd <repository-name>
   ```

2. Build the Docker image using the Dockerfile.

   ```bash
   docker-compose build
   ```

3. Start the services using Docker Compose.

   ```bash
   docker-compose up
   ```

4. The server will be accessible at `http://localhost:8000`.

### Run Locally with Cargo

1. Clone the repository.

   ```bash
   git clone <repository-url>
   cd <repository-name>
   ```

2. Configure the `.env` file with the following:

   ```env
   DATABASE_URL=postgres://<username>:<password>@<host>:<port>/<database>
   ```

3. Run the project locally using Cargo:

   ```bash
   cargo run
   ```

   Ensure the `DATABASE_URL` environment variable is set before running.

---


