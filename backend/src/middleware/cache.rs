use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    Error,
};
use actix_web::middleware::Next;

/// Simple cache control middleware that adds Cache-Control headers to GET responses
/// For a full caching implementation, consider using actix-web's built-in cache middleware
/// or a separate caching layer like Redis
pub struct CacheMiddleware;

impl CacheMiddleware {
    pub fn new() -> Self {
        Self
    }

    pub async fn cache_middleware(
        req: ServiceRequest,
        next: Next<impl MessageBody>,
    ) -> Result<ServiceResponse<impl MessageBody>, Error> {
        // Call the next middleware
        let mut res = next.call(req).await?;

        // Add cache-control headers for GET requests
        if res.request().method() == actix_web::http::Method::GET {
            res.headers_mut().insert(
                actix_web::http::header::CACHE_CONTROL,
                actix_web::http::header::HeaderValue::from_static("public, max-age=300"),
            );
        }

        Ok(res)
    }
}

impl Default for CacheMiddleware {
    fn default() -> Self {
        Self::new()
    }
}
