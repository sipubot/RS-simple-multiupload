# RS-simple-multiupload

멀티파트 업로드 기반의 이미지/JSON 전송 서버입니다. Actix Web를 사용해 파일 업로드와 메타데이터 저장을 지원하며, 정적 이미지 저장 경로를 구성해 간단한 업로드 서비스 형태로 동작합니다.

## 프로젝트 개요

이 프로젝트는 `uploadimage`와 `uploadjson` 엔드포인트를 제공하며, 업로드된 파일을 `static/images` 아래에 저장하고 JSON 형식의 정보를 같이 기록합니다. 웹에서 이미지를 전송하거나, 업로드된 데이터와 함께 별도 메타데이터를 남기고 싶을 때 활용할 수 있습니다.

## 핵심 기능

- multipart/form-data 기반 파일 업로드
- 이미지 저장 경로 자동 생성
- 업로드된 JSON 기록
- 정적 파일 서빙
- 간단한 config 기반 서버 주소 설정

## 실행 방법

1. 의존성을 설치하고 빌드합니다.
   ```bash
   cargo build --release
   ```
2. 서버를 실행합니다.
   ```bash
   cargo run --release
   ```
3. 기본 설정에 따라 `0.0.0.0:9999`에서 서비스가 시작됩니다.

## 주요 엔드포인트

### `POST /uploadimage`
- multipart 파일 업로드
- 업로드 파일을 `static/images/...`에 저장

### `POST /uploadjson/{item_no}/{seq}/{filename}`
- JSON 페이로드를 받아 파일로 저장
- 구조화된 메타데이터 기록

### `GET /static/images`
- 업로드된 자원을 접근할 수 있는 정적 경로 제공

## 구성

```text
RS-simple-multiupload/
├── Cargo.toml
├── README.md
├── src/
│   └── main.rs
├── static/
│   └── images/
└── ...
```

## 기술 스택

- Rust
- Actix Web
- Actix Multipart
- Serde / Serde JSON
- UUID

## 주의사항

- 현재 코드는 실험적이며, 파일 sanitization 로직이 일부 단순화되어 있습니다.
- 서버가 실행될 경로와 실제 업로드 디렉터리가 맞는지 확인해야 합니다.
- 프로덕션 환경에서는 업로드 크기 제한, 인증, 파일 검증, 보안 필터가 추가되어야 합니다.

## 활용 예시

- 이미지 업로드 서비스 프로토타입
- 간단한 CMS/관리자 페이지의 첨부파일 수신
- 내부 테스트용 멀티파트 데이터 수집 서버
