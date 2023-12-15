---
title: p2p-with-tracker v1.0.0
language_tabs:
  - shell: Shell
  - http: HTTP
  - javascript: JavaScript
  - ruby: Ruby
  - python: Python
  - php: PHP
  - java: Java
  - go: Go
toc_footers: []
includes: []
search: true
code_clipboard: true
highlight_theme: darkula
headingLevel: 2
generator: "@tarslib/widdershins v4.0.17"

---

# p2p-with-tracker

> v1.0.0

Base URLs:

* <a href="http://127.0.0.1:48349/api/v1">开发环境: http://127.0.0.1:48349/api/v1(随机生成)</a>

* <a href="http://127.0.0.1:38080/api/v1">测试环境: http://127.0.0.1:38080/api/v1</a>

# Authentication

# Default

## POST tracker心跳

POST /heart_beat

> Body 请求参数

```json
{
  "node_id": "string",
  "addr": "string"
}
```

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|body|body|object| 否 |none|
|» node_id|body|string| 是 |none|
|» addr|body|string| 是 |none|

> 返回示例

> 200 Response

```json
{
  "code": "string",
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|成功|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» code|string|true|none||none|
|» msg|string|true|none||none|

## POST 启动node下载

POST /start_download

> Body 请求参数

```json
{
  "filename": "string"
}
```

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|body|body|object| 否 |none|
|» filename|body|string| 是 |none|

> 返回示例

> 200 Response

```json
{
  "code": "string",
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|成功|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» code|string|true|none||none|
|» msg|string|true|none||none|

## POST 获取node中的分片

POST /fetch_piece

> Body 请求参数

```json
{
  "piece_id": "string"
}
```

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|body|body|object| 否 |none|
|» piece_id|body|string| 是 |none|

> 返回示例

> 200 Response

```json
{
  "code": "string",
  "msg": "string",
  "data": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|成功|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» code|string|true|none||none|
|» msg|string|true|none||none|
|» data|string|true|none||none|

## GET 获取tracker信息

GET /info

> Body 请求参数

```json
{}
```

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|body|body|object| 否 |none|

> 返回示例

> 200 Response

```json
{
  "code": "string",
  "msg": "string",
  "session_info": "string",
  "node_info": "string",
  "piece_info": "string",
  "piece_node_info": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|成功|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» code|string|true|none||none|
|» msg|string|true|none||none|
|» session_info|string|true|none||none|
|» node_info|string|true|none||none|
|» piece_info|string|true|none||none|
|» piece_node_info|string|true|none||none|

## POST 获取分片的node列表

POST /node_list

> Body 请求参数

```json
{
  "piece_id": "string"
}
```

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|body|body|object| 否 |none|
|» piece_id|body|string| 是 |none|

> 返回示例

> 200 Response

```json
{
  "code": "string",
  "msg": "string",
  "node_list": [
    "string"
  ]
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|成功|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» code|string|true|none||none|
|» msg|string|true|none||none|
|» node_list|[string]|true|none||none|

## POST 更新分片信息

POST /report

> Body 请求参数

```json
{
  "piece_id": "string",
  "node_id": "string",
  "progress": "string",
  "filename": "string"
}
```

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|body|body|object| 否 |none|
|» piece_id|body|string| 是 |none|
|» node_id|body|string| 是 |none|
|» progress|body|string| 是 |none|
|» filename|body|string| 是 |none|

> 返回示例

> 200 Response

```json
{
  "code": "string",
  "msg": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|成功|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» code|string|true|none||none|
|» msg|string|true|none||none|

## POST 获取seed信息

POST /fetch_seed

> Body 请求参数

```json
{
  "filename": "string"
}
```

### 请求参数

|名称|位置|类型|必选|说明|
|---|---|---|---|---|
|body|body|object| 否 |none|
|» filename|body|string| 是 |none|

> 返回示例

> 200 Response

```json
{
  "code": "string",
  "msg": "string",
  "seed_info": "string"
}
```

### 返回结果

|状态码|状态码含义|说明|数据模型|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|成功|Inline|

### 返回数据结构

状态码 **200**

|名称|类型|必选|约束|中文名|说明|
|---|---|---|---|---|---|
|» code|string|true|none||none|
|» msg|string|true|none||none|
|» seed_info|string|true|none||none|
