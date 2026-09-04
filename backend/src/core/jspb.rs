use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;

/****
 * Module: JSPB Responder (Google's JavaScript Protocol Buffers / Anti-Hijacking JSON)
 *
 * Chức năng:
 * - Tự động mã hóa Struct/Tuple thành JSON.
 * - Chèn tiền tố `)]}',\n` ngay trước body để chống JSON Hijacking (CVE-2006-0215).
 * - Nếu muốn ép JSON thành Mảng (Array) để tiết kiệm băng thông giống hệt JSPB,
 *   bạn chỉ cần truyền vào một Tuple Struct thay vì Named Struct!
 *
 * Ví dụ Tuple Struct: `pub struct UserArray(pub u64, pub String);`
 ****/
pub struct Jspb<T>(pub T);

impl<T> IntoResponse for Jspb<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match serde_json::to_string(&self.0) {
            Ok(json_body) => {
                // Tiền tố kinh điển của Google
                let body = format!(")]}}',\n{}", json_body);

                (
                    [(header::CONTENT_TYPE, "application/json")],
                    body,
                )
                    .into_response()
            }
            Err(e) => {
                error!(category = "Error", "Lỗi Serialize JSPB: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Lỗi hệ thống khi mã hóa JSPB",
                )
                    .into_response()
            }
        }
    }
}
