# Cleopatra
- [Introduction](#introduction)
- [Technical Stack](#technical-stack)
- [Database Design](#database-design)
- [API Design](#api-design)


## Introduction

The backend service built by Rust to test result management - view, report, analysis.

For this system, my goal was to keep the application as lightweight as possible while ensuring functionality and performance. <br/>

Therefore, I developed it using Rust and Sqlite3. <br/>

Sqlite3 combined with WAL ensures low maintenance costs while improving write performance. This is enough fora team of fewer than 50 engineers. <br/>

Rust's extreme performance low memory requirements, and type safety makes it ideal for streaming, high-concurrency scenarios.

## Technical Stack

1. Build the service with Rust Axum (Axum version 0.7)

2. Database is Sqlite3 + WAL

3. Project structure

```text
cleopatra/
├── Cargo.toml
├── Makefile
├── migrations/
│   └── cleopatra.sql
└── src/
    ├── main.rs
    ├── db.rs  // init db configuration here
    ├── models.rs // define models
    └── routes/
        ├── mod.rs // define axum routes
        ├── execution.rs // define restful execution api
        ├── result.rs // define resrful test result api
        └── stream.rs // define stream api
```

## Database Design

We use Sqlite3 + WAL to avoid write/read condition

### Table - test_result

The table represent test result.

| column | type | comment |
|----------|----------|----------|
| id    | INTEGER, AUTOINCREMENT   | The primary key    |
| name    | VARCHAR(32) NOT NULL     | the test case name     |
| platform    | VARCHAR(32) NOT NULL     | possiable value - api, web, android, ios, etc     |
| description    | VARCHAR(128)     | the description of test case     |
| status    | CHAR (2)  NOT NULL  | test status, P -> Pass, F -> Fail, I -> Ignored |
| execution_time    | INTEGER     | the time of test execution |
| counter    | INTEGER     | how many times to run this test |
| log    | Text     | the log of test cases     |
| execution_id    | INTEGER  NOT NULL   | represent which execution the test belongs to       |
| screenshot_id    | INTEGER     | the id of screenshot, this is used to get the screenshot via other service     |
| created_by    | VARCHAR(32)      | the user who run the test     |
| time_created   | INTEGER NOT NULL     | time created |

### Table - execution

The table which represent a set of test result

| column | type | comment |
|----------|----------|----------|
| id    | INTEGER , AUTOINCREMENT    | the primary key  |
| name    | VARCHAR(32)      | the name of execution     |
| tag    | VARCHAR(64)      | the tag of execution     |
| created_by    | VARCHAR(32)      | the user who trigger the execution |
| time_created   | INTEGER     | time created |


## API Design

We have two kind API.

1. Restful

2. Html Stream

### Restful

| api | description |  success http status |
|----------|----------|----------|
| [POST /api/execution](#post-apiexecution)  | create a execution | 201 |
| [GET /api/executions](#get-apiexecutions) | get executions by criteria| 200 |
| [GET /api/execution/{id}/result](#get-apiexecutionidtests)  | get all of tests by execution id, excluding log field | 200 |
| [POST /api/result](#post-apitest)  | publish a test result | 201 |
| [GET /api/result](#get-apiresultid)  | get test result by id | 200 |

#### POST /api/execution

request payload

```json
{
  "name": "login regression suite",
  "tag": "release_2025_09",
  "created_by": "alice"
}
```

response payload

```json
{
  "id": 101,
  "name": "login regression suite",
  "tag": "release_2025_09",
  "created_by": "alice",
  "time_created": 1736900000
}
```

#### Get /api/executions

get executions by criteria

| parameter           | type     | comment                |
| ------------ | ------ | ----------------- |
| `created_by` | string | filter by created_by, do not support fuzzy matching           |
| `name`       | string | filter by name, support fuzzy matching   |
| `tag`        | string | filter by tag, do not support fuzzy matching             |
| `limit`      | int    | the count per page，default is 20, max is 100             |
| `offset`     | int    | pagination offset, default 0     |


sample request
```textmate
GET /api/executions?created_by=alice&limit=20&offset=0

GET /api/executions?name=login&limit=10&offset=10
```

response payload
```json
{
  "total": 52,             // total count
  "limit": 20,             // the current limit
  "offset": 0,             // the current offset
  "has_next": true,        // whether we have next
  "items": [
    {
      "id": 101,
      "name": "login regression suite",
      "tag": "release_2025_09",
      "created_by": "alice",
      "time_created": 1736900000
    },
    {
      "id": 104,
      "name": "login e2e",
      "tag": "release_2025_09",
      "created_by": "alice",
      "time_created": 1736900600
    }
  ]
}

```

#### Get /api/execution/{id}/results

| parameter         | type     | comment                                             |
| ---------- | ------ | ----------------------------------------------- |
| `status`   | string | filter by status, F/P/I                         |
| `platform` | string | filter by platform（mutiple value，api/android/ios/web, etc） |
| `limit`    | int    | the count per page，default is 20, max is 100                             |
| `offset`   | int    | pagination offset, default 0                                  |
| `include_summary`   | boolean    | whether compute the summary and show it in response, default false                                 |

response
```json
{
  "execution_id": 123,
  "summary": {
    "total": 3, // total test in this execution
    "pass": 1, // total passed test
    "fail": 1, // total failed test
    "ignor": 1 // total ignore test
  },
  "total": 52,
  "limit": 20,
  "offset": 0,
  "has_next": true,
  "items": [
    {
      "id": 1001,
      "name": "login test",
      "platform": "web",
      "description": "login page should work",
      "status": "P",
      "execution_time": 2000,
      "screenshot_id": 1,
      "counter": 1,
      "created_by": "alice",
      "time_created": 1736900000
    },
    {
      "id": 1002,
      "name": "signup test",
      "platform": "android",
      "description": "signup flow",
      "status": "F",
      "execution_time": 3500,
      "screenshot_id": null,
      "counter": 2,
      "created_by": "alice",
      "time_created": 1736900010
    }
  ]
}
```



#### POST /api/result

Fristly, this api checks wheter test reulst already exist by execution_id and test name.
1. If test resut doesn't exist, create it in Table test. the counter is set to 1
2. Otherwise, update it and increase the counter in test table.

request
```json
{
  "execution_id": 101,
  "name": "login_with_valid_user",
  "platform": "web",
  "description": "verify login with valid account",
  "status": "P",
  "execution_time": 523,
  "log": "ok",
  "screenshot_id": 201,
  "created_by": "alice",
  "time_created": 1736900000
}
```

#### GET /api/result/{id}

response

```json
{
  "id": 1001,
  "execution_id": 123,
  "name": "login test",
  "platform": "web",
  "description": "login page should work",
  "status": "P",
  "execution_time": 2000,
  "log": "Test started...\nLogin page opened...\nAssertion passed.",
  "screenshot_id": 1,
  "created_by": "alice",
  "time_created": 1736900000
}
```



###  Html Stream API

#### POST  /api/executions/{execution_id}/results:stream

This is used to create a long connection so that client can continuesly publish test result server.

It works like Restful POST - /api/test publish a test result, which depends on whether test result exists by execution_id and test name to save or update the test result.

1. POST /api/result/stream

2. Request format - NDJSON

```textmate
http header:
```textmate
Content-Type: application/x-ndjson
Transfer-Encoding: chunked
```

3. Request payload

```textmate
{"name": "login_with_valid_user", "platform": "web", "description": "verify login with valid account", "status": "P", "execution_time": 523, "log": "ok", "screenshot_id": 201, "created_by": "alice", "time_created": 1736900000}
{"name": "login_with_invalid_user", "platform": "web", "description": "verify login with invalid account", "status": "F", "execution_time": 341, "log": "Invalid password", "screenshot_id": 202, "created_by": "alice", "time_created": 1736900003}
```

4. Response

All of the test result are persisted
```json
{
  "status": "C",
  "message": "Some test results failed",
  "received": 100,
  "inserted": 100
}
```

Partial test results are persisted
```json
{
  "status": "F",
  "message": "Some test results failed",
  "execution_id": 123,
  "received": 3,
  "inserted": 2,
  "failed": 1,
  "failed_items": [
    {
      "test_name": "invalid status test",
      "error": "Invalid status value: X",
      "raw_payload": {
        "name": "invalid status test",
        "platform": "ios",
        "status": "X",
        "execution_time": 1000,
        "log": "wrong status",
        "created_by": "alice",
        "time_created": 1736900020
      }
    }
  ]
}

```

All of the test results are failed to be persisted
```json
{
  "status": "F",
  "message": "Some test results failed",
  "execution_id": 123,
  "received": 3,
  "inserted": 2,
  "failed": 1,
  "failed_items": [
    {
      "test_name": "invalid status test",
      "error": "Invalid status value: X",
      "raw_payload": {
        "name": "invalid status test",
        "platform": "ios",
        "status": "X",
        "execution_time": 1000,
        "log": "wrong status",
        "created_by": "alice",
        "time_created": 1736900020
      }
    }
  ]
}
```

### API Error Handling Response

No matter restful and html stream api, it should follow same convenstion to process exception.

1. For 4xx error, response sample:

```json
{
  "error": "ValidationError",
  "message": "Invalid status value: X",
  "field": "status"
}
```

2. For 500 error, response sample:

```json
{
  "error": "InternalError",
  "message": "Database write failed"
}
```



